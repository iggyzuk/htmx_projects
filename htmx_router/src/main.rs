use axum::{http::Method, routing::get, Router};

use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

mod constants;
mod handlers;
mod hx;
mod markup;
mod state;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let hero = Router::new()
        .route("/", get(handlers::hero))
        .route("/description", get(handlers::hero_description))
        .route("/abilities", get(handlers::hero_abilities))
        .route("/talents", get(handlers::hero_talents))
        .route("/ability/:ability", get(handlers::ability))
        .route("/talent/:talent", get(handlers::talent));

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/heroes", get(handlers::heroes))
        .route("/abilities", get(handlers::abilities))
        .route("/talents", get(handlers::talents))
        .route("/about", get(handlers::about))
        .nest("/:hero", hero)
        .layer(cors)
        .with_state(AppState::new());

    let address = "0.0.0.0:4204";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
