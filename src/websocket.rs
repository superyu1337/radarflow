use std::{net::SocketAddr, sync::Arc};
use futures_util::{SinkExt, StreamExt};

use tokio::{net::{TcpListener, TcpStream}, sync::RwLock};
use tokio_tungstenite::{accept_async, tungstenite::{Error, Message}};

use anyhow::Result;

use crate::structs::{Config, communication::RadarData};

async fn handle_connection(peer: SocketAddr, stream: TcpStream, data_lock: Arc<RwLock<RadarData>>) -> Result<(), Error> {
    let mut ws_stream = accept_async(stream).await?;

    log::info!("New WebSocket connection: {}", peer);

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;

        if msg.is_close() {
            ws_stream.close(None).await?;
        }

        if let Message::Text(str) = msg {

            if str == "requestInfo" {
                let data = data_lock.read().await;

                match serde_json::to_string(&*data) {
                    Ok(json) => {
                        ws_stream.send(Message::Text(json)).await?;
                    },
                    Err(e) => {
                        log::error!("Could not serialize data into json: {}", e.to_string());
                        log::error!("Sending \"error\" instead");
                        ws_stream.send(Message::Text(String::from("error"))).await?;
                    },
                }
            }
        }
    }

    Ok(())
}


async fn accept_connection(peer: SocketAddr, stream: TcpStream, data_lock: Arc<RwLock<RadarData>>) {
    if let Err(e) = handle_connection(peer, stream, data_lock).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => log::error!("Error processing connection: {}", err),
        }
    }
}

pub async fn run(config: Config, data_lock: Arc<RwLock<RadarData>>) -> anyhow::Result<()> {

    let address = format!("0.0.0.0:{}", config.websocket_port());
    let listener = TcpListener::bind(&address).await.expect("Can't listen");
    log::info!("TcpListener bound to {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        log::info!("Peer connected from {}", peer);

        tokio::spawn(accept_connection(peer, stream, data_lock.clone()));
    }

    Ok(())
}
