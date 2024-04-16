use anyhow::{Context, Result};
use axum::{
    extract::Query,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_htmx::HxResponseTrigger;
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let env_status = match dotenvy::from_filename("htmx_events/.env") {
        Ok(_) => "found local .env",
        Err(_) => "no local .env",
    };
    tracing::info!(env_status);

    let app = Router::new()
        .route("/", get(index))
        .route("/trigger", post(trigger))
        .route("/time", get(time))
        .route("/time-with-event", get(time_with_event))
        .route("/validate", get(validate))
        .layer(CorsLayer::permissive());

    let address = "0.0.0.0:4209";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("failed to bind TcpListener")?;

    tracing::info!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> Markup {
    base(html! {

        // Polling request every second
        p hx-trigger="load, every 1s" hx-get="/time" {
            "..."
        }

        // Sequence of events
        button hx-post="/trigger" {
            "Press Me!"
        }
        h2 hx-trigger="button-pressed from:body" hx-get="/time-with-event" {
            "..."
        }
        h3 hx-trigger="time-updated from:body" hx-get="/time" {
            "..."
        }

        // Input with validation
        input
        name="text"
        class="form-control"
        placeholder="Some text here"
        type="text"
        hx-target="#validation"
        hx-get="/validate"
        hx-trigger="keyup changed delay:500ms" // removing this will validate on un-focus
        {}

        div #validation {}
    })
}

async fn trigger() -> impl IntoResponse {
    (HxResponseTrigger::normal(["button-pressed"]), "Boom!")
}

async fn time() -> impl IntoResponse {
    chrono::Utc::now().format("%H:%M:%S").to_string()
}

async fn time_with_event() -> impl IntoResponse {
    (
        HxResponseTrigger::normal(["time-updated"]),
        chrono::Utc::now().format("%H:%M:%S").to_string(),
    )
}

#[derive(Deserialize)]
struct ValidateQuery {
    text: String,
}

async fn validate(Query(form): Query<ValidateQuery>) -> impl IntoResponse {
    if form.text.is_empty() {
        return html! {};
    }

    let long = form.text.len() > 6;
    let val_text = if long { "looks good!" } else { "too short" };

    html! {
        div .alert .alert-success[long] .alert-danger[!long] .my-2 {
            b { (val_text) } " " small { "\"" (form.text) "\"" }
        }
    }
}

pub(crate) fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-bs-theme="light" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Events (htmx)" }

                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}

                script src="https://unpkg.com/htmx.org@1.9.10" {}
            }
            body {
                div ."container d-flex justify-content-center p-2" {
                    div style="width:400px;" {
                        h1 { "Events (htmx)" }
                        (content)
                    }
                }
            }
        }
    }
}
