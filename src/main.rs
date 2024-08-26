use chc_service::telemetry::initialize_tracing_subscriber;
use chc_service::LocalChcServerCli;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    initialize_tracing_subscriber("info");

    let cli_args = LocalChcServerCli::parse();
    let chc_service = cli_args.create_chc_service()?;

    let address = chc_service.address();

    tracing::info!("Starting service on http://{}", address);
    chc_service.run().await?;

    Ok(())
}
