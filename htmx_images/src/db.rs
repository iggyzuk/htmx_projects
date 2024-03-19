use crate::state::{AppState, Image};
use std::error::Error;

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
    image_data: &Vec<u8>,
) -> Result<Image, Box<dyn Error>> {
    const QUERY: &'static str = r#"
INSERT INTO image (file_name, mime_type, image_data)
VALUES ($1, $2, $3)
RETURNING id, file_name, mime_type, image_data, created_at;
    "#;

    let database = &state.database;

    let img = sqlx::query_as(QUERY)
        .bind(&file_name)
        .bind(&mime_type)
        .bind(image_data)
        .fetch_one(database)
        .await?;

    Ok(img)
}
