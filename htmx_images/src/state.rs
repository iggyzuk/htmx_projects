use std::sync::Arc;

use serde::Deserialize;
use sqlx::{postgres::PgPool, prelude::FromRow};

pub(crate) type AppState = Arc<State>;

pub(crate) struct State {
    pub(crate) database: PgPool,
}

impl State {
    pub(crate) fn new(database: PgPool) -> AppState {
        Arc::new(Self { database })
    }
}

#[derive(Deserialize, FromRow)]
pub(crate) struct Image {
    pub(crate) id: i64,
    pub(crate) file_name: String,
    pub(crate) mime_type: String,
    pub(crate) image_data: Vec<u8>,
    pub(crate) dominant_color: Option<i32>,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
}

impl Image {
    pub(crate) fn as_base64(&self) -> String {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::Engine;

        STANDARD_NO_PAD.encode(&self.image_data)
    }

    pub(crate) fn src(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.as_base64())
    }

    pub(crate) fn short_date(&self) -> String {
        self.created_at.format("%y-%m-%d").to_string()
    }
}
