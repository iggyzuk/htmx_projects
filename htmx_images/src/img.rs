use std::error::Error;

use image::{
    codecs::gif::GifDecoder, imageops::FilterType, AnimationDecoder, DynamicImage, Frame,
    GenericImageView, ImageDecoder, RgbaImage,
};
use webp::{AnimDecoder, AnimEncoder, AnimFrame, Encoder, WebPConfig};

use crate::mime;

const TARGET_PX: u32 = 150;
const QUALITY: f32 = 70.0;
const MAX_FRAMES: usize = 24;
const FRAME_RATE: i32 = 100;

pub(crate) fn thumbnail_for_mime(
    data: &[u8],
    mime_type: &String,
) -> Result<(Vec<u8>, String, i32), Box<dyn Error>> {
    match mime_type.as_str() {
        "image/webp" => webp_to_webp(&data),
        "image/gif" => gif_to_webp(&data),
        _ => any_to_webp(&data),
    }
}

pub(crate) fn any_to_webp(bytes: &[u8]) -> Result<(Vec<u8>, String, i32), Box<dyn Error>> {
    let img = image::load_from_memory(bytes)?;
    let small_img = resize_and_crop(img);
    let dominant = dominant(&small_img.to_rgb8());
    let encoder = Encoder::from_image(&small_img)?;
    let webp_memory = encoder.encode(QUALITY);
    Ok((webp_memory.to_vec(), mime::IMAGE_WEBP.to_string(), dominant))
}

pub(crate) fn gif_to_webp(gif_bytes: &[u8]) -> Result<(Vec<u8>, String, i32), Box<dyn Error>> {
    let gif_decoder = GifDecoder::new(gif_bytes)?;

    let (original_width, original_height) = gif_decoder.dimensions();

    let gif_frames: Vec<Frame> = gif_decoder.into_frames().collect::<Result<_, _>>()?;

    // Keep only 24 evenly distributed frames.
    let frames_count = gif_frames.len();
    let frame_step = frames_count / std::cmp::min(MAX_FRAMES, frames_count);

    // Temporary store all scaled and resized frames here,
    // so that in the next step we can construct a webp.
    let mut processed_frames = vec![];

    // Scale and resize all gif frames.
    for gif_frame in gif_frames.into_iter().step_by(frame_step) {
        let gif_frame_bytes: Vec<u8> = gif_frame.into_buffer().into_vec();
        let img = RgbaImage::from_vec(original_width, original_height, gif_frame_bytes).unwrap();
        processed_frames.push(resize_and_crop(img.into()));
    }

    // Construct WebP animation encoder with lossy config.
    let mut config = WebPConfig::new().map_err(|_| "could not create webp config")?;
    config.lossless = 0;
    config.alpha_compression = 0;
    config.quality = QUALITY;

    let mut anim_encoder = AnimEncoder::new(TARGET_PX, TARGET_PX, &config);
    anim_encoder.set_loop_count(0);

    // Copy all frames into WebP.
    let mut timestamp = 0;
    for frame in &processed_frames {
        let anim = AnimFrame::from_image(frame, timestamp)?;
        anim_encoder.add_frame(anim);
        timestamp += FRAME_RATE;
    }

    // Take the dominant color from the first processed frame.
    let dominant = {
        let first_frame = &processed_frames[0];
        dominant(&first_frame.to_rgb8())
    };

    // Write bytes from encoder.
    let webp_encoder = anim_encoder.encode();
    Ok((
        webp_encoder.to_vec(),
        mime::IMAGE_WEBP.to_string(),
        dominant,
    ))
}

pub(crate) fn webp_to_webp(bytes: &[u8]) -> Result<(Vec<u8>, String, i32), Box<dyn Error>> {
    let anim_image = AnimDecoder::new(&bytes).decode()?;

    if anim_image.has_animation() {
        let mut processed_frames = vec![];

        let frames: Vec<AnimFrame> = anim_image.into_iter().collect();

        let frames_count = frames.len();
        let frame_step = frames_count / std::cmp::min(MAX_FRAMES, frames_count);

        for frame in frames.into_iter().step_by(frame_step) {
            let img =
                RgbaImage::from_vec(frame.width(), frame.height(), frame.get_image().to_vec())
                    .unwrap();
            processed_frames.push(resize_and_crop(img.into()))
        }

        // Construct WebP animation encoder with lossy config.
        let mut config = WebPConfig::new().map_err(|_| "could not create webp config")?;
        config.lossless = 0;
        config.alpha_compression = 0;
        config.quality = QUALITY;

        let mut anim_encoder = AnimEncoder::new(TARGET_PX, TARGET_PX, &config);
        anim_encoder.set_loop_count(0); // Set loop count to 0 for infinite loop, change if needed

        // Copy all frames into WebP.
        let mut timestamp = 0;
        for frame in &processed_frames {
            let anim = AnimFrame::from_image(frame, timestamp)?;
            anim_encoder.add_frame(anim);
            timestamp += FRAME_RATE;
        }

        // Take the dominant color from the first processed frame.
        let dominant = {
            let first_frame = &processed_frames[0];
            dominant(&first_frame.to_rgb8())
        };

        // Write bytes from encoder.
        let webp_encoder = anim_encoder.encode();
        return Ok((
            webp_encoder.to_vec(),
            mime::IMAGE_WEBP.to_string(),
            dominant,
        ));
    }

    any_to_webp(bytes)
}

fn resize_and_crop(img: DynamicImage) -> DynamicImage {
    let (window_width, window_height) = window(img.width(), img.height());

    let dyn_img: DynamicImage = img.to_rgba8().into();
    let resized_dyn_img = dyn_img.resize(window_width, window_height, FilterType::Lanczos3);

    // get the half offset by doing: 190 - 150 = 40 -> 40 / 2 = 20
    let center_x = (window_width - TARGET_PX) / 2;
    let center_y = (window_height - TARGET_PX) / 2;

    // Crop image at the center
    resized_dyn_img
        .view(center_x, center_y, TARGET_PX, TARGET_PX)
        .to_image()
        .into()
}

// Calculate the new dimensions while maintaining the aspect ratio
fn window(width: u32, height: u32) -> (u32, u32) {
    if width > height {
        let aspect = width as f32 / height as f32;
        ((TARGET_PX as f32 * aspect).round() as u32, TARGET_PX)
    } else {
        let aspect = height as f32 / width as f32;
        (TARGET_PX, (TARGET_PX as f32 * aspect).round() as u32)
    }
}

/// Find the dominant color from the given image as bytes
/// # Important
/// Bytes must be in RGB format!
fn dominant(bytes: &[u8]) -> i32 {
    let palette = color_thief::get_palette(&bytes, color_thief::ColorFormat::Rgb, 10, 3).unwrap();

    let dominant = palette[0];

    let r = (dominant.r as u32) << 16;
    let g = (dominant.g as u32) << 8;
    let b = dominant.b as u32;

    (r | g | b) as i32
}
