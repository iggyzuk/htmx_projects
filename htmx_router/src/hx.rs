use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

/// A struct for the htmx request which can be used
/// as an extractor in axum handle functions.
///
/// ```rust
/// async fn handler(hx_req: HxReq) -> Response {
///     if hx_req.is_targeting("special-id") {
///         fragment
///     } else {
///         markup::body(fragment)
///     }
/// }
/// ```
pub(crate) struct HxReq {
    pub(crate) request: Option<String>,
    pub(crate) target: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for HxReq
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self {
            request: parts
                .headers
                .get("hx-request")
                .map(|x| x.to_str().unwrap().to_string()),
            target: parts
                .headers
                .get("hx-target")
                .map(|x| x.to_str().unwrap().to_string()),
        })
    }
}

impl HxReq {
    pub(crate) fn has_request(&self) -> bool {
        self.request.is_some()
    }

    /// We can only send a fragment back when the
    /// original request was targeting a specific id.
    pub(crate) fn is_targeting<T: AsRef<str>>(&self, target: T) -> bool {
        self.has_request() && self.target.as_ref().is_some_and(|t| t == target.as_ref())
    }
}
