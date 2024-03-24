//! Example OAuth (Google) implementation.

use anyhow::{anyhow, Context, Result};
use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Query, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    RequestPartsExt, Router,
};
use axum_extra::{headers, typed_header::TypedHeaderRejectionReason, TypedHeader};
use http::{header, request::Parts, StatusCode};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, RevocationUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

mod markup;

static COOKIE_NAME: &str = "SESSION";
static CSRF_TOKEN: &str = "csrf_token";

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let env_status = match dotenvy::from_filename("htmx_auth/.env") {
        Ok(_) => "found local .env",
        Err(_) => "no local .env",
    };
    tracing::info!(env_status);

    // `MemoryStore` is just used as an example. Don't use this in production.
    let store = MemoryStore::new();
    let oauth_client = oauth_client().unwrap();
    let app_state = AppState {
        store,
        oauth_client,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/google", get(google_auth))
        .route("/auth/authorized", get(login_authorized))
        .route("/protected", get(protected))
        .route("/protected-fragment", get(protected_fragment))
        .route("/logout", get(logout))
        .with_state(app_state);

    let address = "0.0.0.0:4206";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("failed to bind TcpListener")?;

    tracing::debug!("🚀 Server Started: {address} 🚀");

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    store: MemoryStore,
    oauth_client: BasicClient,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

fn oauth_client() -> Result<BasicClient, AppError> {
    use std::env::var;

    // Environment variables (* = required):
    // *"GOOGLE_CLIENT_ID"     "REPLACE_ME";
    // *"GOOGLE_CLIENT_SECRET" "REPLACE_ME";
    //  "REDIRECT_URL"         "http://127.0.0.1:4206/auth/authorized";
    //  "AUTH_URL"             "https://accounts.google.com/o/oauth2/v2/auth";
    //  "TOKEN_URL"            "https://www.googleapis.com/oauth2/v3/token";

    let google_client_id = ClientId::new(
        var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let auth_url =
        AuthUrl::new(var("AUTH_URL").expect("Missing the REDIRECT_URL environment variable."))
            .expect("Invalid authorization endpoint URL");

    let token_url =
        TokenUrl::new(var("TOKEN_URL").expect("Missing the TOKEN_URL environment variable."))
            .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    );

    Ok(client
        .set_redirect_uri(
            RedirectUrl::new(
                var("REDIRECT_URL").expect("Missing the REDIRECT_URL environment variable."),
            )
            .expect("Invalid redirect URL"),
        )
        // Google supports OAuth 2.0 Token Revocation (RFC-7009)
        .set_revocation_uri(
            RevocationUrl::new(
                var("REVOKE_URL").expect("Missing the REVOKE_URL environment variable."),
            )
            .expect("Invalid revocation endpoint URL"),
        ))
}

// The user data we'll get back from Google.
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    email: String,
    name: String,
    verified_email: bool,
    picture: String,
}

// Session is optional
async fn index(user: Option<User>) -> impl IntoResponse {
    markup::home(&user)
}

async fn google_auth(
    State(client): State<BasicClient>,
    State(store): State<MemoryStore>,
) -> Result<impl IntoResponse, AppError> {
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    // Create session to store csrf_token
    let mut session = Session::new();
    session
        .insert(CSRF_TOKEN, &csrf_token)
        .context("failed in inserting CSRF token into session")?;

    // Store the session in MemoryStore and retrieve the session cookie
    let cookie = store
        .store_session(session)
        .await
        .context("failed to store CSRF token session")?
        .context("unexpected error retrieving CSRF cookie value")?;

    // Attach the session cookie to the response header
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; Path=/");
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().context("failed to parse cookie")?,
    );

    Ok((headers, Redirect::to(auth_url.as_ref())))
}

// Valid user session required. If there is none, redirect to the auth page
async fn protected(user: User) -> impl IntoResponse {
    markup::protected(&user)
}

// No valid user session is required, will not try to redirect to auth page.
async fn protected_fragment(user: Option<User>) -> impl IntoResponse {
    if let Some(user) = user {
        return markup::protected(&user).into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

async fn logout(
    State(store): State<MemoryStore>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<impl IntoResponse, AppError> {
    let cookie = cookies
        .get(COOKIE_NAME)
        .context("unexpected error getting cookie name")?;

    let session = match store
        .load_session(cookie.to_string())
        .await
        .context("failed to load session")?
    {
        Some(s) => s,
        // No session active, just redirect
        None => return Ok(Redirect::to("/")),
    };

    store
        .destroy_session(session)
        .await
        .context("failed to destroy session")?;

    Ok(Redirect::to("/"))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthRequest {
    code: String,
    state: String,
}

async fn csrf_token_validation_workflow(
    auth_request: &AuthRequest,
    cookies: &headers::Cookie,
    store: &MemoryStore,
) -> Result<(), AppError> {
    // Extract the cookie from the request
    let cookie = cookies
        .get(COOKIE_NAME)
        .context("unexpected error getting cookie name")?
        .to_string();

    // Load the session
    let session = match store
        .load_session(cookie)
        .await
        .context("failed to load session")?
    {
        Some(session) => session,
        None => return Err(anyhow!("Session not found").into()),
    };

    // Extract the CSRF token from the session
    let stored_csrf_token = session
        .get::<CsrfToken>(CSRF_TOKEN)
        .context("CSRF token not found in session")?
        .to_owned();

    // Cleanup the CSRF token session
    store
        .destroy_session(session)
        .await
        .context("Failed to destroy old session")?;

    // Validate CSRF token is the same as the one in the auth request
    if *stored_csrf_token.secret() != auth_request.state {
        return Err(anyhow!("CSRF token mismatch").into());
    }

    Ok(())
}

async fn login_authorized(
    Query(query): Query<AuthRequest>,
    State(store): State<MemoryStore>,
    State(oauth_client): State<BasicClient>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> Result<impl IntoResponse, AppError> {
    csrf_token_validation_workflow(&query, &cookies, &store).await?;

    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .context("failed in sending request request to authorization server")?;

    // Fetch user data from google
    let client = reqwest::Client::new();
    let user_data: User = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .context("failed in sending request to target Url")?
        .json::<User>()
        .await
        .context("failed to deserialize response as JSON")?;

    // Create a new session filled with user data
    let mut session = Session::new();
    session
        .insert("user", &user_data)
        .context("failed in inserting serialized value into session")?;

    // Store session and get corresponding cookie
    let cookie = store
        .store_session(session)
        .await
        .context("failed to store session")?
        .context("unexpected error retrieving cookie value")?;

    // Build the cookie
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; Path=/");

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        cookie.parse().context("failed to parse cookie")?,
    );

    Ok((headers, Redirect::to("/")))
}

struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth/google").into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = AuthRedirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => AuthRedirect,
                    _ => panic!("unexpected error getting Cookie header(s): {e}"),
                },
                _ => panic!("unexpected error getting cookies: {e}"),
            })?;
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(AuthRedirect)?;

        let user = session.get::<User>("user").ok_or(AuthRedirect)?;

        Ok(user)
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
