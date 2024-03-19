use std::error::Error;
use std::io::BufWriter;
use std::io::Cursor;
use std::io::Write;

use image::codecs::gif::GifDecoder;
use image::imageops::FilterType;
use image::AnimationDecoder;
use image::DynamicImage;
use image::Frame;
use image::GenericImageView;

use image::ImageDecoder;
use image::RgbaImage;

use webp::AnimEncoder;
use webp::AnimFrame;
use webp::WebPConfig;

use crate::mime;

const TARGET_PX: u32 = 150;

pub(crate) fn thumbnail_for_mime(
    data: &[u8],
    mime_type: &String,
) -> Result<(Vec<u8>, String), Box<dyn Error>> {
    if mime_type == &mime::IMAGE_GIF.to_string() {
        animated_webp_thumbnail_from_gif(&data)
    } else {
        webp_thumbnail(&data)
    }
}

pub(crate) fn webp_thumbnail(bytes: &[u8]) -> Result<(Vec<u8>, String), Box<dyn Error>> {
    let img = image::load_from_memory(bytes)?;

    let (original_width, original_height) = img.dimensions();
    let (width, height) = window(original_width, original_height);

    let resized_img = image::imageops::resize(&img, width, height, FilterType::Lanczos3);

    tracing::info!("resized dimensions: {:?}", resized_img.dimensions());

    // get the half offset by doing: 190 - 150 = 40 -> 40 / 2 = 20
    let center_x = (width - TARGET_PX) / 2;
    let center_y = (height - TARGET_PX) / 2;

    // Crop image at the center
    let crop_img = resized_img
        .view(center_x, center_y, TARGET_PX, TARGET_PX)
        .to_image();

    // Create the WebP encoder
    let dynamic_img = DynamicImage::from(crop_img);
    let encoder = webp::Encoder::from_image(&dynamic_img)?;
    let webp = encoder.encode(80.0);

    // Save resized image to a Vec<u8>
    let mut final_bytes = Vec::new();
    {
        let mut buf_writer = BufWriter::new(Cursor::new(&mut final_bytes));
        buf_writer.write(webp.as_ref())?;
    } // buf_writer is dropped

    Ok((final_bytes, mime::IMAGE_WEBP.to_string()))
}

pub(crate) fn animated_webp_thumbnail_from_gif(
    gif_bytes: &[u8],
) -> Result<(Vec<u8>, String), Box<dyn Error>> {
    let gif_decoder = GifDecoder::new(gif_bytes)?;

    let (original_width, original_height) = gif_decoder.dimensions();
    let (width, height) = window(original_width, original_height);

    let gif_frames: Vec<Frame> = gif_decoder.into_frames().collect::<Result<_, _>>()?;

    // Keep only 24 evenly distributed frames.
    let frames_count = gif_frames.len();
    let frame_step = frames_count / std::cmp::min(24, frames_count);
    let gif_frames = gif_frames.into_iter().step_by(frame_step);

    // Temporary store all scaled and resized frames here,
    // so that in the next step we can construct a webp.
    let mut frame_data = vec![];

    // Scale and resize all gif frames.
    for gif_frame in gif_frames {
        let gif_frame_bytes = gif_frame.buffer().to_vec();
        let rgba_img = RgbaImage::from_raw(original_width, original_height, gif_frame_bytes)
            .ok_or("could not create rgba image from raw bytes")?;
        let dyn_img_frame = DynamicImage::from(rgba_img);
        let resized_dyn_img_frame = dyn_img_frame.resize(width, height, FilterType::Lanczos3);

        // get the half offset by doing: 190 - 150 = 40 -> 40 / 2 = 20
        let center_x = (width - TARGET_PX) / 2;
        let center_y = (height - TARGET_PX) / 2;

        // Crop image at the center
        let crop_img = DynamicImage::from(
            resized_dyn_img_frame
                .view(center_x, center_y, TARGET_PX, TARGET_PX)
                .to_image(),
        );
        frame_data.push(crop_img);
    }

    // Construct WebP animation encoder with lossy config.
    let mut config = WebPConfig::new().map_err(|_| "could not create webp config")?;
    config.lossless = 0;
    config.alpha_compression = 0;
    config.quality = 70.0;

    let mut anim_encoder = AnimEncoder::new(TARGET_PX, TARGET_PX, &config);
    anim_encoder.set_loop_count(0); // Set loop count to 0 for infinite loop, change if needed

    // Copy all frames into WebP.
    let mut timestamp = 0;
    for frame in &frame_data {
        let anim = AnimFrame::from_image(frame, timestamp)?;
        anim_encoder.add_frame(anim);
        timestamp += 100;
    }

    // Write bytes from encoder.
    let webp_encoder = anim_encoder.encode();
    let mut final_bytes = Vec::new();
    {
        let mut buf_writer = BufWriter::new(&mut final_bytes);
        buf_writer.write(&webp_encoder)?;
    } // buf_writer is dropped

    Ok((final_bytes, mime::IMAGE_WEBP.to_string()))
}

// Calculate the new dimensions while maintaining the aspect ratio
fn window(width: u32, height: u32) -> (u32, u32) {
    if width > height {
        let aspect = width as f32 / height as f32;
        ((TARGET_PX as f32 * aspect) as u32, TARGET_PX)
    } else {
        let aspect = height as f32 / width as f32;
        (TARGET_PX, (TARGET_PX as f32 * aspect) as u32)
    }
}
