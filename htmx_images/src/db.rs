use std::error::Error;

use crate::state::{AppState, Image};

pub(crate) async fn get_image(state: &AppState, id: i64) -> Result<Image, Box<dyn Error>> {
    const QUERY: &'static str = r#"SELECT * FROM image WHERE id = $1;"#;
    let database = &state.database;
    let img = sqlx::query_as(QUERY).bind(id).fetch_one(database).await?;
    Ok(img)
}

pub(crate) async fn get_all_images(state: &AppState) -> Result<Vec<Image>, Box<dyn Error>> {
    const QUERY: &'static str = r#"
SELECT * FROM image
ORDER BY created_at DESC
LIMIT 100;
"#;
    let database = &state.database;
    let img = sqlx::query_as(QUERY).fetch_all(database).await?;
    Ok(img)
}

pub(crate) async fn insert_image(
    state: &AppState,
    file_name: String,
    mime_type: String,
    image_data: &[u8],
    dominant_color: i32,
) -> Result<Image, Box<dyn Error>> {
    const QUERY: &'static str = r#"
INSERT INTO image (file_name, mime_type, image_data, dominant_color)
VALUES ($1, $2, $3, $4)
RETURNING *;
    "#;

    let database = &state.database;

    let img = sqlx::query_as(QUERY)
        .bind(&file_name)
        .bind(&mime_type)
        .bind(image_data)
        .bind(dominant_color)
        .fetch_one(database)
        .await?;

    Ok(img)
}
