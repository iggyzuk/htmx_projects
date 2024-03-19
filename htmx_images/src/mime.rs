use mediatype::names::GIF;
use mediatype::names::IMAGE;
use mediatype::names::WEBP;
use mediatype::MediaType;

pub(crate) const IMAGE_GIF: MediaType = MediaType::new(IMAGE, GIF);
pub(crate) const IMAGE_WEBP: MediaType = MediaType::new(IMAGE, WEBP);
