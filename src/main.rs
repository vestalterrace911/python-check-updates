mod cli;
mod config;
mod output;
mod parsers;
mod pypi;
mod self_update;
mod uninstall;
mod upgrade;
mod version;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
