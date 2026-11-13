//! Embedded protocol registry. Every command a human (trusted chrome) or an
//! agent (MCP) can run is declared here with its permission category.

use serde_json::{Map, Value};
use std::sync::{Arc, OnceLock};

pub const REGISTRY_JSON: &str = include_str!("../registry/justslide.protocol.v1.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exposure {
    /// Trusted browser chrome only (human actor).
    Ui,
    /// Authenticated agents over MCP/IPC, subject to grants.
    Mcp,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub name: String,
    pub mcp_tool: String,
    pub title: String,
    pub description: String,
    /// Permission category (`read.metadata`, `write.organize`, ...), `human`
    /// for chrome-only commands, or `None` for unauthenticated-safe reads
    /// such as `capabilities.get`.
    pub operation: Option<String>,
    pub idempotency: String,
    pub activates: bool,
    pub exposure: Vec<Exposure>,
    pub arguments: Value,
    pub result: Value,
}

impl CommandSpec {
    pub fn exposed_to(&self, e: Exposure) -> bool {
        self.exposure.contains(&e)
    }
    pub fn is_read_only(&self) -> bool {
        matches!(self.operation.as_deref(), None | Some("read.metadata" | "read.content" | "read.capture"))
            || self.idempotency == "safe"
    }
}

#[derive(Debug)]
pub struct Registry {
    pub raw: Value,
    pub version: String,
    pub commands: Vec<CommandSpec>,
}

impl Registry {
    pub fn get() -> &'static Registry {
        static R: OnceLock<Registry> = OnceLock::new();
        R.get_or_init(|| Registry::parse(REGISTRY_JSON).expect("embedded protocol registry is valid"))
    }

    pub fn parse(text: &str) -> anyhow::Result<Registry> {
        let raw: Value = serde_json::from_str(text)?;
        let defs = raw["definitions"].clone();
        let mut commands = Vec::new();
        for c in raw["commands"].as_array().ok_or_else(|| anyhow::anyhow!("commands missing"))? {
            let s = |k: &str| c[k].as_str().unwrap_or_default().to_string();
            let exposure = c["exposure"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| match x.as_str() {
                            Some("ui") => Some(Exposure::Ui),
                            Some("mcp") => Some(Exposure::Mcp),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_else(|| vec![Exposure::Ui]);
            commands.push(CommandSpec {
                name: s("name"),
                mcp_tool: s("mcpTool"),
                title: s("title"),
                description: s("description"),
                operation: c["operation"].as_str().map(str::to_string),
                idempotency: s("idempotency"),
                activates: c["activates"].as_bool().unwrap_or(false),
                exposure,
                arguments: inline_refs(&c["arguments"], &defs, 0)?,
                result: inline_refs(&c["result"], &defs, 0)?,
            });
        }
        Ok(Registry { version: raw["version"].as_str().unwrap_or_default().to_string(), raw, commands })
    }

    pub fn by_tool(&self, tool: &str) -> Option<&CommandSpec> {
        self.commands.iter().find(|c| c.mcp_tool == tool)
    }

    pub fn by_command(&self, name: &str) -> Option<&CommandSpec> {
        self.commands.iter().find(|c| c.name == name)
    }

    pub fn mcp_commands(&self) -> impl Iterator<Item = &CommandSpec> {
        self.commands.iter().filter(|c| c.exposed_to(Exposure::Mcp))
    }

    pub fn protocol_name(&self) -> &str {
        self.raw["protocol"].as_str().unwrap_or_default()
    }

    pub fn max_frame_bytes(&self) -> usize {
        self.raw["framing"]["maxFrameBytes"].as_u64().unwrap_or(65536) as usize
    }

    pub fn max_result_frame_bytes(&self) -> usize {
        self.raw["framing"]["maxResultFrameBytes"].as_u64().unwrap_or(8 << 20) as usize
    }

    pub fn request_timeout_ms(&self) -> u64 {
        self.raw["limits"]["requestTimeoutMs"].as_u64().unwrap_or(30_000)
    }

    pub fn limit(&self, key: &str, default: u64) -> u64 {
        self.raw["limits"][key].as_u64().unwrap_or(default)
    }
}

fn inline_refs(v: &Value, defs: &Value, depth: usize) -> anyhow::Result<Value> {
    anyhow::ensure!(depth < 16, "schema $ref nesting too deep");
    Ok(match v {
        Value::Object(m) => {
            if let Some(Value::String(r)) = m.get("$ref") {
                let key = r.strip_prefix("#/definitions/").ok_or_else(|| anyhow::anyhow!("bad $ref {r}"))?;
                let target = defs.get(key).ok_or_else(|| anyhow::anyhow!("unknown $ref {r}"))?;
                return inline_refs(target, defs, depth + 1);
            }
            let mut out = Map::new();
            for (k, val) in m {
                out.insert(k.clone(), inline_refs(val, defs, depth + 1)?);
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| inline_refs(x, defs, depth + 1)).collect::<Result<_, _>>()?),
        other => other.clone(),
    })
}

pub fn as_json_object(v: &Value) -> Arc<Map<String, Value>> {
    Arc::new(v.as_object().cloned().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_registry_parses_and_inlines_refs() {
        let r = Registry::get();
        assert_eq!(r.version, "1.1.0");
        assert!(r.commands.len() >= 40);
        let create = r.by_tool("justslide_tabs_create").unwrap();
        assert_eq!(create.name, "tabs.create");
        assert!(!create.arguments.to_string().contains("$ref"));
        assert_eq!(create.arguments["properties"]["activate"]["const"], Value::Bool(false));
        for c in &r.commands {
            assert!(c.mcp_tool.starts_with("justslide_"), "{}", c.mcp_tool);
            assert!(c.mcp_tool.len() <= 64);
            assert!(c.mcp_tool.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_'), "{}", c.mcp_tool);
        }
    }

    #[test]
    fn activating_and_human_commands_are_never_exposed_to_agents() {
        for c in Registry::get().mcp_commands() {
            assert!(!c.activates, "{} activates but is MCP-exposed", c.name);
            assert_ne!(c.operation.as_deref(), Some("human"), "{} is human-only but MCP-exposed", c.name);
            assert!(!c.name.starts_with("grants.") && !c.name.starts_with("approvals.") && !c.name.starts_with("agents."),
                "{}: agents must not manage their own grants or approvals", c.name);
        }
    }

    #[test]
    fn command_names_are_unique() {
        let mut names: Vec<_> = Registry::get().commands.iter().map(|c| c.name.clone()).collect();
        let n = names.len();
        names.sort();
        names.dedup();
        assert_eq!(n, names.len());
    }
}
