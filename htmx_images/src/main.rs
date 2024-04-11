use std::{error::Error, path::PathBuf};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};

use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, services::ServeDir};

mod db;
mod handler;
mod img;
mod markup;
mod mime;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("should have CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest_dir);

    let mut env_path = manifest_path.clone();
    env_path.push(".env");
    let _ = dotenvy::from_filename(&env_path);

    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // database connection
    let pg_url = std::env::var("DATABASE_URL")?;
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&pg_url)
        .await?;

    // migrations
    sqlx::migrate!().run(&pg_pool).await?;

    let mut assets_path = manifest_path.clone();
    assets_path.push("assets");

    let serve_dir_service = ServeDir::new(assets_path);

    let app = Router::new()
        .route("/", get(handler::index))
        .route("/images", get(handler::images))
        .route("/images/:id", get(handler::image))
        .route("/images/:id/modal", get(handler::image_modal))
        .route("/images", post(handler::upload_image))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024)) // 100 MB
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .nest_service("/assets", serve_dir_service)
        .with_state(state::State::new(pg_pool));

    let address = "0.0.0.0:4205";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
