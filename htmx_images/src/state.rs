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
        use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};

        STANDARD_NO_PAD.encode(&self.image_data)
    }

    pub(crate) fn src(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.as_base64())
    }

    pub(crate) fn short_date(&self) -> String {
        self.created_at.format("%y-%m-%d").to_string()
    }

    pub(crate) fn dominant_hex(&self, alpha: f32) -> String {
        assert!(
            alpha >= 0.0 && alpha <= 1.0,
            "alpha should be between 0 and 1"
        );

        let color_int = self.dominant_color.unwrap_or_default();

        let red = (color_int >> 16) & 0xFF;
        let green = (color_int >> 8) & 0xFF;
        let blue = color_int & 0xFF;
        let alpha = (alpha * 255.0) as i32;

        format!("#{:02X}{:02X}{:02X}{:02X}", red, green, blue, alpha)
    }
}
