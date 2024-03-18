use std::error::Error;

use axum::{
    extract::DefaultBodyLimit,
    http::Method,
    routing::{get, post},
    Router,
};

use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
};

mod db;
mod handler;
mod img;
mod markup;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let env_status = match dotenvy::from_filename("htmx_images/.env") {
        Ok(_) => "found local .env",
        Err(_) => "no local .env",
    };
    tracing::info!(env_status);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // database connection
    let pg_url = std::env::var("DATABASE_URL")?;
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&pg_url)
        .await?;

    // migrations
    sqlx::migrate!().run(&pg_pool).await?;

    let app = Router::new()
        .route("/", get(handler::index))
        .route("/images", get(handler::images))
        .route("/images/:id", get(handler::image))
        .route("/images", post(handler::upload_image))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024)) // 100 MB
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state::State::new(pg_pool));

    let address = "0.0.0.0:4205";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
