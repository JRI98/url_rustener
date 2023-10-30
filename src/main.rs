// URL shortener with stats and automatic deletion if unused for 7 days

mod server;

use std::env;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use server::{AppState, Routes};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let shared_state = AppState::new().expect("Could not create shared state");

    let app = Router::new()
        .route("/:slug", get(Routes::get_url))
        .route("/:slug/stats", get(Routes::get_url_stats))
        .route("/", post(Routes::create_url))
        .route("/:slug", patch(Routes::update_url))
        .route("/:slug", delete(Routes::delete_url))
        .with_state(shared_state);

    let port = env::var("PORT").unwrap_or("3000".to_string());

    axum::Server::bind(
        &format!("0.0.0.0:{port}")
            .parse()
            .expect("Could not parse address"),
    )
    .serve(app.into_make_service())
    .await
    .unwrap();
}
