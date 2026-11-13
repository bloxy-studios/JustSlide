use anyhow::{Context, Result, bail};
use justslide_mcp::endpoint::{self, TokenSource};
use justslide_protocol::Registry;
use justslide_mcp::server::{Bridge, BridgeConfig, CLIENT_NAME_DEFAULT};
use rmcp::ServiceExt;
use std::path::PathBuf;

const USAGE: &str = "justslide-mcp [--user-data-dir DIR] [--token-file FILE | --keychain-account NAME] [--client-name NAME] [--print-tools] [--doctor]

Local stdio MCP bridge for a running JustSlide app (WebKit edition). stdout carries MCP
protocol only; diagnostics go to stderr (RUST_LOG=info|debug).";

fn main() -> Result<()> {
    let mut user_data_dir: Option<PathBuf> = std::env::var_os("JUSTSLIDE_USER_DATA_DIR").map(PathBuf::from);
    let mut token_file: Option<PathBuf> = std::env::var_os("JUSTSLIDE_TOKEN_FILE").map(PathBuf::from);
    let mut keychain_account: Option<String> = None;
    let mut client_name = CLIENT_NAME_DEFAULT.to_string();
    let mut print_tools = false;
    let mut doctor = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--user-data-dir" => user_data_dir = it.next().map(PathBuf::from),
            "--token-file" => token_file = it.next().map(PathBuf::from),
            "--keychain-account" => keychain_account = it.next(),
            "--client-name" => client_name = it.next().context("--client-name needs a value")?,
            "--print-tools" => print_tools = true,
            "--doctor" => doctor = true,
            "-h" | "--help" => { eprintln!("{USAGE}"); return Ok(()); }
            "--version" => { eprintln!("justslide-mcp {} (protocol {})", env!("CARGO_PKG_VERSION"), Registry::get().version); return Ok(()); }
            other => bail!("unknown argument {other}\n{USAGE}"),
        }
    }
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .init();

    if print_tools {
        let tools: Vec<_> = Registry::get().commands.iter().map(|c| serde_json::json!({"tool": c.mcp_tool, "command": c.name, "operation": c.operation})).collect();
        eprintln!("{}", serde_json::to_string_pretty(&tools)?);
        return Ok(());
    }
    let user_data_dir = match user_data_dir { Some(d) => d, None => endpoint::default_user_data_dir()? };
    let token = match (token_file, keychain_account) {
        (Some(f), _) => TokenSource::File(f),
        (None, Some(a)) => TokenSource::Keychain { account: a },
        (None, None) => TokenSource::Keychain { account: client_name.clone() },
    };
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async move {
        if doctor {
            let reg = Registry::get();
            let ep = endpoint::discover(&user_data_dir, reg.raw["protocol"].as_str().unwrap_or_default())?;
            eprintln!("endpoint ok: instance {} socket {}", ep.record.instance_id, ep.record.socket.display());
            endpoint::read_token(&token)?;
            eprintln!("pairing token ok (value not shown)");
            return Ok(());
        }
        let bridge = Bridge::new(BridgeConfig { user_data_dir, token, client_name });
        let service = bridge.serve(rmcp::transport::stdio()).await.context("MCP handshake on stdio failed")?;
        service.waiting().await?;
        Ok(())
    })
}
