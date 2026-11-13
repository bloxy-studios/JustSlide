//! Minimal JSON-Schema subset validator for the keywords used by the registry
//! (type, required, properties, additionalProperties=false, enum, const,
//! pattern, minLength/maxLength, minimum/maximum, items). The browser process
//! re-validates every request; this is an early, bounded rejection in the
//! bridge so malformed calls never reach the browser.

use regex::Regex;
use serde_json::Value;

pub fn validate(schema: &Value, value: &Value) -> Result<(), String> {
    check(schema, value, "$", 0)
}

fn check(s: &Value, v: &Value, path: &str, depth: usize) -> Result<(), String> {
    if depth > 32 {
        return Err(format!("{path}: nesting too deep"));
    }
    if let Some(c) = s.get("const") {
        if c != v {
            return Err(format!("{path}: must equal {c}"));
        }
    }
    if let Some(Value::Array(options)) = s.get("enum") {
        if !options.contains(v) {
            return Err(format!("{path}: must be one of {}", Value::Array(options.clone())));
        }
    }
    if let Some(Value::String(t)) = s.get("type") {
        let ok = match t.as_str() {
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "boolean" => v.is_boolean(),
            "integer" => v.as_i64().is_some() || v.as_u64().is_some(),
            "number" => v.is_number(),
            _ => true,
        };
        if !ok {
            return Err(format!("{path}: expected {t}"));
        }
    }
    if let Value::String(text) = v {
        let len = text.chars().count() as u64;
        if let Some(min) = s.get("minLength").and_then(Value::as_u64) {
            if len < min { return Err(format!("{path}: shorter than {min}")); }
        }
        if let Some(max) = s.get("maxLength").and_then(Value::as_u64) {
            if len > max { return Err(format!("{path}: longer than {max}")); }
        }
        if let Some(Value::String(p)) = s.get("pattern") {
            let re = Regex::new(p).map_err(|e| format!("{path}: bad pattern: {e}"))?;
            if !re.is_match(text) { return Err(format!("{path}: does not match {p}")); }
        }
    }
    if let Some(n) = v.as_f64() {
        if let Some(min) = s.get("minimum").and_then(Value::as_f64) {
            if n < min { return Err(format!("{path}: below minimum {min}")); }
        }
        if let Some(max) = s.get("maximum").and_then(Value::as_f64) {
            if n > max { return Err(format!("{path}: above maximum {max}")); }
        }
    }
    if let (Some(items), Value::Array(arr)) = (s.get("items"), v) {
        for (i, item) in arr.iter().enumerate() {
            check(items, item, &format!("{path}[{i}]"), depth + 1)?;
        }
    }
    if let Value::Object(obj) = v {
        let props = s.get("properties").and_then(Value::as_object);
        if let Some(Value::Array(req)) = s.get("required") {
            for r in req.iter().filter_map(Value::as_str) {
                if !obj.contains_key(r) { return Err(format!("{path}.{r}: required")); }
            }
        }
        for (k, val) in obj {
            match props.and_then(|p| p.get(k)) {
                Some(ps) => check(ps, val, &format!("{path}.{k}"), depth + 1)?,
                None if s.get("additionalProperties") == Some(&Value::Bool(false)) => {
                    return Err(format!("{path}.{k}: unexpected property"));
                }
                None => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;
    use serde_json::json;

    fn args(tool: &str) -> Value { Registry::get().by_tool(tool).unwrap().arguments.clone() }

    #[test]
    fn accepts_valid_background_tab() {
        let a = json!({"workspaceId": "ws_agent0001", "url": "https://example.com/", "pinned": true});
        validate(&args("justslide_tabs_create"), &a).unwrap();
    }

    #[test]
    fn rejects_activation_and_non_http_urls_and_extra_fields() {
        let s = args("justslide_tabs_create");
        assert!(validate(&s, &json!({"workspaceId": "ws_agent0001", "url": "https://e.com", "activate": true})).is_err());
        assert!(validate(&s, &json!({"workspaceId": "ws_agent0001", "url": "javascript:alert(1)"})).is_err());
        assert!(validate(&s, &json!({"workspaceId": "ws_agent0001", "url": "file:///etc/passwd"})).is_err());
        assert!(validate(&s, &json!({"workspaceId": "ws_agent0001", "url": "chrome://settings"})).is_err());
        assert!(validate(&s, &json!({"workspaceId": "ws_agent0001", "url": "https://e.com", "actor": "root"})).is_err());
        assert!(validate(&s, &json!({"workspaceId": "../../etc", "url": "https://e.com"})).is_err());
    }

    #[test]
    fn bounds_are_enforced() {
        let s = args("justslide_page_screenshot");
        assert!(validate(&s, &json!({"tabId": "tab_abcdefgh", "maxEdgePx": 99999})).is_err());
        validate(&s, &json!({"tabId": "tab_abcdefgh", "maxEdgePx": 800})).unwrap();
    }
}
