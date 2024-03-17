use std::error::Error;
use std::io::BufWriter;
use std::io::Cursor;

use image::imageops::FilterType;
use image::GenericImageView;
use image::ImageFormat;

const TARGET: u32 = 150;

pub(crate) fn make_jpeg_thumbnail(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let img = image::load_from_memory(bytes)?;

    // Calculate the new dimensions while maintaining the aspect ratio
    let (mut width, mut height) = img.dimensions();

    if width > height {
        let aspect = width as f32 / height as f32;
        width = (TARGET as f32 * aspect) as u32;
        height = TARGET;
    } else {
        let aspect = height as f32 / width as f32;
        width = TARGET;
        height = (TARGET as f32 * aspect) as u32;
    }

    let resized_img = image::imageops::resize(&img, width, height, FilterType::Lanczos3);

    tracing::info!("resized dimensions: {:?}", resized_img.dimensions());

    // get the half offset by doing: 190 - 150 = 40 -> 40 / 2 = 20
    let center_x = (width - TARGET) / 2;
    let center_y = (height - TARGET) / 2;

    let crop_img = resized_img.view(center_x, center_y, 150, 150).to_image();

    // Save resized image to a Vec<u8>
    let mut resized_image_bytes = Vec::new();
    {
        let mut buf_writer = BufWriter::new(Cursor::new(&mut resized_image_bytes));
        crop_img.write_to(&mut buf_writer, ImageFormat::Jpeg)?
    }

    Ok(resized_image_bytes)
}
