use std::error::Error;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{db, img, markup, mime, state::AppState};

pub(crate) async fn index() -> maud::Markup {
    crate::markup::home()
}

pub(crate) async fn image(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let res = db::get_image(&state, id).await;
    let img = match res {
        Ok(img) => {
            tracing::info!("getting images {id}");
            img
        }
        Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    };
    (StatusCode::OK, markup::image(&img)).into_response()
}

pub(crate) async fn images(State(state): State<AppState>) -> Response {
    let res = db::get_all_images(&state).await;
    let images = match res {
        Ok(images) => {
            tracing::info!("getting all images");
            images
        }
        Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    };
    (StatusCode::OK, markup::images(&images)).into_response()
}

pub(crate) async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    tracing::info!("uploading image...");

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        let x = format!(
            "Length of `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
            data.len()
        );

        tracing::info!("{}", &x);

        if !content_type.contains(mediatype::names::IMAGE.as_str()) {
            return (StatusCode::BAD_REQUEST, "only images can be uploaded").into_response();
        }

        let (bytes, mime_type) = match img::thumbnail_for_mime(&data, &content_type) {
            Ok(bytes) => bytes,
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }
        };

        let res = db::insert_image(&state, file_name.clone(), mime_type, &bytes).await;

        let image = match res {
            Ok(image) => {
                tracing::info!("inserted image {file_name} into database");
                image
            }
            Err(err) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
        };

        return markup::image(&image).into_response();
    }

    // Something went wrong.
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "could not upload the file".to_string(),
    )
        .into_response()
}
