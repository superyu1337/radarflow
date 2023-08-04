use std::sync::Arc;

use structs::Config;
use tokio::sync::RwLock;

use crate::structs::communication::RadarData;

mod structs;
mod websocket;
mod webserver;
mod dma;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var("RADARFLOW_LOG").is_err() {
        std::env::set_var("RADARFLOW_LOG", "warn")
    }

    env_logger::Builder::from_env("RADARFLOW_LOG")
        .default_format()
        .format_target(false)
        .format_module_path(true)
        .format_timestamp(None)
        .init();


    let config = Config::from_file("./Config.toml")?;
    let web_port = config.web_port();
    let rwlock = Arc::new(
        RwLock::new(
            RadarData::empty()
        )
    );

    let config_clone = config.clone();
    let rwlock_clone = rwlock.clone();
    let dma_handle = tokio::spawn(async move {
        dma::run(config_clone, rwlock_clone).await
    });

    let config_clone = config.clone();
    tokio::spawn(async move {
        let future = websocket::run(config, rwlock);
        future.await
    });

    tokio::spawn(async move {
        let future = webserver::run(config_clone);
        future.await
    });

    // Sleep to print in proper order
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    if let Ok(my_local_ip) = local_ip_address::local_ip() {
        let address = format!("http://{}:{}", my_local_ip, web_port);
        println!("Launched at {}", address);
    } else {
        let address = format!("http://0.0.0.0:{}", web_port);
        println!("launched at! {}", address);
    }

    match dma_handle.await {
        Ok(res) => {
            if let Err(e) = res {
                log::error!("{}", e.to_string());
            }
        }
        Err(err) => {
            log::error!("Error when waiting for dma thread: {}", err.to_string());
        }
    }

    Ok(())
}