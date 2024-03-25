use anyhow::{Context, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use lettre::message::{header, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;

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

    let app = Router::new()
        .route("/", get(index))
        .route("/send-email", post(send_email));

    let address = "0.0.0.0:4207";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("failed to bind TcpListener")?;

    tracing::info!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app).await?;

    Ok(())
}

fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html data-bs-theme="light" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Email (htmx)" }

                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.1/font/bootstrap-icons.css";
                script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}

                script src="https://unpkg.com/htmx.org@1.9.10" {}
            }
            body {
                (content)
            }
        }
    }
}

async fn index() -> Markup {
    base(html! {
        div ."container d-flex justify-content-center p-2" {
            div style="width:400px;" {
                div .card {
                    div .card-body {

                        h1 .card-title { i ."bi bi-envelope-fill" { } " Email (htmx)" }

                        p .card-text {
                            "Send an email with htmx"
                        }

                        p { small .card-text .text-secondary { "Made by " a href="https://iggyzuk.com/" { "Iggy Zuk" } } }

                        h2 { "Preview" }
                        div ."p-2 mb-2 border rounded" {
                            (email_markup(&"content".to_string()))
                        }

                        form hx-post="send-email" hx-disabled-elt="#sub-btn" autocomplete="off" {

                            div class="form-floating mb-3" {
                                input type="email" class="form-control" id="email-input" name="destination" {}
                                label for="email-input" { "Email address" }
                            }

                            div class="form-floating mb-3" {
                                input class="form-control" id="email-subject" name="subject" {}
                                label for="email-subject" { "Email subject" }
                            }

                            div class="form-floating" {
                                textarea class="form-control" id="email-body" name="body" style="height: 100px" {}
                                label for="email-body" { "Email body" }
                            }

                            button #sub-btn ."btn btn-lg btn-primary mt-2 p-2 w-100" { i ."bi bi-envelope-arrow-up-fill" { } " Send" }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct EmailForm {
    subject: String,
    destination: String,
    body: String,
}

async fn send_email(Form(form): Form<EmailForm>) -> Result<impl IntoResponse, AppError> {
    use std::env::var;

    tracing::debug!("sending an email");

    let email = Message::builder()
        .from(var("SMPT_FROM").context("missing SMPT_FROM")?.parse()?)
        .to(form
            .destination
            .parse()
            .context("could not parse destination email")?)
        .subject(form.subject)
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(String::from(email_markup(&form.body))),
        )
        .unwrap();

    let creds = Credentials::new(
        var("SMPT_USERNAME").context("missing SMTP_USERNAME")?,
        var("SMPT_PASSWORD").context("missing SMTP_PASSWORD")?,
    );

    // Open a remote connection to host
    let mailer = SmtpTransport::relay(var("SMTP_HOST").context("missing SMTP_HOST")?.as_ref())?
        .port(
            var("SMTP_PORT")
                .context("missing SMTP_PORT")?
                .parse()
                .context("could not parse SMTP_PORT as a number")?,
        )
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email).context("could not send the email")?;

    tracing::debug!("email sent!");

    Ok(html! { i ."bi bi-envelope-check-fill" { } " Email sent!" }.into_response())
}

// Create the html we want to send.
fn email_markup(body: &String) -> Markup {
    html! {
        head {
            style type="text/css" {
                "h1 { font-family: 'Comic Sans MS', 'Comic Sans', cursive; }"
            }
        }
        div style="display: flex; flex-direction: column; align-items: center;" {
            h1 { "Welcome" }
            p { (body) }
            p { small { "This email was sent with Lettre, a mailer library for Rust!" } }
        }
    }
}

// Use anyhow, define error and enable '?'
// For a simplified example of using anyhow in axum check /examples/anyhow-error-response
#[derive(Debug)]
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Application error: {:#}", self.0);

        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
