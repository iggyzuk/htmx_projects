use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

mod markup;

#[derive(Clone, Deserialize)]
struct Perk {
    name: &'static str,
    desc: &'static str,
    icon: &'static str,
}

#[derive(Clone)]
struct Perks(Vec<Perk>);

impl Perks {
    fn filter(&self, term: &String) -> Vec<&Perk> {
        let mut result = vec![];

        if term.is_empty() {
            return result;
        }

        let term = term.to_lowercase();
        for perk in &self.0 {
            if perk.name.to_lowercase().contains(term.as_str())
                || perk.desc.to_lowercase().contains(term.as_str())
            {
                result.push(perk)
            }
        }
        result
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let env_status = match dotenvy::from_filename("htmx_email/.env") {
        Ok(_) => "found local .env",
        Err(_) => "no local .env",
    };
    tracing::info!(env_status);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let perks: Vec<Perk> = match ron::from_str(include_str!("../content.ron")) {
        Ok(data) => data,
        Err(e) => {
            panic!("error deserializing RON: {}", e);
        }
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/search", post(search))
        .layer(cors)
        .with_state(Perks(perks));

    let address = "0.0.0.0:4208";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("failed to bind TcpListener")?;

    tracing::info!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index(State(state): State<Perks>) -> impl IntoResponse {
    markup::base(markup::search_form(&state.0))
}

#[derive(Deserialize)]
struct Search {
    term: String,
}

async fn search(State(state): State<Perks>, Form(form): Form<Search>) -> impl IntoResponse {
    markup::search_perk_rows(state.filter(&form.term))
}
