use neuronguard::chat_server::router_with_routed_backend;
use neuronguard::expert_router::ExpertRouter;
use neuronguard::lake_config::load_expert_registry;
use neuronguard::local_backend::LlamaCppSubprocessBackend;
use std::env;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let lake_path = env::args()
        .nth(1)
        .or_else(|| env::var("LAKE_CONFIG").ok())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("example/neuronlake_mvp/lake.yaml"));
    let llama_bin = env::var("LLAMA_CPP_BIN").unwrap_or_else(|_| "llama-cli".to_string());
    let extra_args = env::var("LLAMA_CPP_ARGS")
        .ok()
        .map(|args| args.split_whitespace().map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();

    let registry = load_expert_registry(&lake_path)?;
    let expert_router = ExpertRouter::train(&registry)?;
    let backend = Arc::new(LlamaCppSubprocessBackend::new(llama_bin).with_extra_args(extra_args));
    let app = router_with_routed_backend(registry.clone(), expert_router, backend);

    let host: IpAddr = registry.server().host.parse()?;
    let address = SocketAddr::new(host, registry.server().port);
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!(
        "NeuronLake MVP serving model '{}' at http://{}/v1/chat/completions",
        registry.server().model_name,
        address
    );
    println!("Loaded experts:");
    for expert in registry.experts() {
        println!("  - {}: {}", expert.id, expert.domain);
    }

    axum::serve(listener, app).await?;
    Ok(())
}
