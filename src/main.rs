mod server;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use server::{AppState, Routes};
use std::env;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let redis_url = &env::var("REDIS_URL").unwrap_or("redis://127.0.0.1".to_string());
    let shared_state = AppState::new(redis_url).expect("Could not create shared state");

    let app = Router::new()
        .route("/:slug", get(Routes::get_url))
        .route("/:slug/stats", get(Routes::get_url_stats))
        .route("/", post(Routes::create_url))
        .route("/:slug", patch(Routes::update_url))
        .route("/:slug", delete(Routes::delete_url))
        .with_state(shared_state);

    let port = env::var("PORT").unwrap_or("3000".to_string());

    axum::Server::bind(
        &format!("[::]:{port}")
            .parse()
            .expect("Could not parse address"),
    )
    .serve(app.into_make_service())
    .await
    .expect("The server stopped");
}
