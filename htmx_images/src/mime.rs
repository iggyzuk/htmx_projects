use mediatype::{
    names::{GIF, IMAGE, WEBP},
    MediaType,
};

pub(crate) const IMAGE_GIF: MediaType = MediaType::new(IMAGE, GIF);
pub(crate) const IMAGE_WEBP: MediaType = MediaType::new(IMAGE, WEBP);
