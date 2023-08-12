use std::sync::Arc;
use clap::Parser;
use cli::Cli;

use tokio::sync::RwLock;

use crate::structs::communication::RadarData;

mod structs;
mod webserver;
mod dma;
mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let cli = Cli::parse();

    if std::env::var("RADARFLOW_LOG").is_err() {
        std::env::set_var("RADARFLOW_LOG", "warn")
    }

    simple_logger::SimpleLogger::new()
        .with_level(cli.loglevel.into())
        .init()
        .expect("Initializing logger");

    let rwlock = Arc::new(
        RwLock::new(
            RadarData::empty()
        )
    );

    let rwlock_clone = rwlock.clone();
    let dma_handle = tokio::spawn(async move {
        dma::run(cli.connector, cli.pcileech_device, cli.poll_rate, rwlock_clone).await
    });

    tokio::spawn(async move {
        let future = webserver::run(cli.web_path, cli.port, rwlock);

        if let Ok(my_local_ip) = local_ip_address::local_ip() {
            let address = format!("http://{}:{}", my_local_ip, cli.port);
            println!("Launched webserver at {}", address);
        } else {
            let address = format!("http://0.0.0.0:{}", cli.port);
            println!("launched webserver at! {}", address);
        }
    
        future.await
    });

    if let Err(err) = dma_handle.await {
        log::error!("Error when waiting for dma thread: {}", err.to_string());
    }

    Ok(())
}