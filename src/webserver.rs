use axum::Router;
use tower_http::services::ServeDir;

use crate::structs::Config;

pub async fn run(config: Config) -> anyhow::Result<()> {
    let app = Router::new()
        .nest_service("/", ServeDir::new(config.web_path()));

    let address = format!("0.0.0.0:{}", config.web_port());

    axum::Server::bind(&address.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}