use crate::im;
use image::{GenericImage, Rgb};
use std::path::PathBuf;

pub fn run(target: &PathBuf, intensity_levels: u32) {
    for (image, info) in im::open_and_setup_output(target) {
        process_image(image, info, intensity_levels)
    }
}

fn process_image(image: image::DynamicImage, info: im::ImageInfo, intensity_levels: u32) {
    im::image_buffer_save_png(
        image_rotate_45(image.clone()),
        &info.save_path_concat("rotate_45", image::ImageFormat::Png),
    );
    im::image_buffer_save_png(
        image_rotate_90(image.clone()),
        &info.save_path_concat("rotate_90", image::ImageFormat::Png),
    );

    let mut factor: u8 = 1;
    for level in 0..intensity_levels {
        if factor == 128 {
            break;
        }
        factor *= 2;
        im::image_buffer_save_png(
            image_clamp_intensity_level(image.clone(), factor),
            &info.save_path_concat(&format!("intensity_level_{level}"), image::ImageFormat::Png),
        );
    }

    let block_mean_sizes = [3, 11, 21];
    for block_size in block_mean_sizes {
        im::image_buffer_save_png(
            image_set_pixels_to_block_mean(image.clone(), block_size),
            &info.save_path_concat(
                &format!("pixels_to_block_mean_{block_size}x{block_size}"),
                image::ImageFormat::Png,
            ),
        );
    }

    let region_mean_sizes = [3, 5, 7];
    for block_size in region_mean_sizes {
        im::image_buffer_save_png(
            image_set_region_to_block_mean(image.clone(), block_size),
            &info.save_path_concat(
                &format!("region_to_block_mean_{block_size}x{block_size}"),
                image::ImageFormat::Png,
            ),
        );
    }
}

fn rotated_dimensions(width: u32, height: u32) -> (u32, u32) {
    let diagonal = ((width as f64).powf(2.0) + (height as f64).powf(2.0)).sqrt();
    let new_width = diagonal.round() as u32;
    let new_height = new_width;
    (new_width, new_height)
}

fn image_rotate_45(image: image::DynamicImage) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    use imageproc::geometric_transformations::{rotate_about_center, Interpolation};

    let buffer = image.into_rgb8();
    //let new_width = ((buffer.width() as f32).abs() * 2f32.sqrt()).ceil() as u32;
    //let new_height = ((buffer.height() as f32).abs() * 2f32.sqrt()).ceil() as u32;
    let (new_width, new_height) = rotated_dimensions(buffer.width(), buffer.height());
    let x_offset = (new_width - buffer.width()) / 2;
    let y_offset = (new_height - buffer.height()) / 2;

    let mut rotated_image = image::ImageBuffer::new(new_width, new_height);
    rotated_image
        .copy_from(&buffer, x_offset, y_offset)
        .expect("rotate copy from");

    rotate_about_center(
        &rotated_image,
        std::f32::consts::PI / 4.0,
        Interpolation::Bilinear,
        Rgb([0, 0, 0]),
    )
}

fn image_rotate_90(image: image::DynamicImage) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    image.rotate90().into_rgb8()
}

fn image_clamp_intensity_level(
    image: image::DynamicImage,
    factor: u8,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut buffer = image.into_rgb8();
    for (_, _, pixel) in buffer.enumerate_pixels_mut() {
        pixel[0] = pixel[0] / factor * factor;
        pixel[1] = pixel[1] / factor * factor;
        pixel[2] = pixel[2] / factor * factor;
    }
    buffer
}

fn image_set_pixels_to_block_mean(
    image: image::DynamicImage,
    block_size: u32,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    assert!(
        block_size % 2 == 1,
        "block_size must be odd (eg: 1, 3, 5, 7..)"
    );
    let src_buffer = image.into_rgb8();
    let offset = block_size / 2;

    let mut buffer = src_buffer.clone();
    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
        let x_lb = if x <= offset { 0 } else { x - offset };
        let x_rb = (x + offset).clamp(x, src_buffer.width() - 1);
        let y_tb = if y <= offset { 0 } else { y - offset };
        let y_bb = (y + offset).clamp(y, src_buffer.height() - 1);

        let mut color_sum: [u32; 3] = [0, 0, 0];
        let mut area_count: u32 = 0;

        for xs in x_lb..=x_rb {
            for ys in y_tb..=y_bb {
                let src_pixel = src_buffer.get_pixel(xs, ys);
                color_sum[0] += src_pixel[0] as u32;
                color_sum[1] += src_pixel[1] as u32;
                color_sum[2] += src_pixel[2] as u32;
                area_count += 1;
            }
        }
        let mean = Rgb::<u8>([
            (color_sum[0] / area_count) as u8,
            (color_sum[1] / area_count) as u8,
            (color_sum[2] / area_count) as u8,
        ]);

        *pixel = mean;
    }
    buffer
}

fn image_set_region_to_block_mean(
    image: image::DynamicImage,
    block_size: u32,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    assert!(
        block_size % 2 == 1,
        "block_size must be odd (eg: 1, 3, 5, 7..)"
    );
    let src_buffer = image.into_rgb8();
    let mut buffer = src_buffer.clone();

    let x_block_count = buffer.width() / block_size;
    let y_block_count = buffer.height() / block_size;

    for xb in 0..x_block_count {
        for yb in 0..y_block_count {
            let x_lb = xb * block_size;
            let x_rb = (x_lb + block_size).clamp(0, src_buffer.width() - 1);
            let y_tb = yb * block_size;
            let y_bb = (y_tb + block_size).clamp(0, src_buffer.height() - 1);

            let mut color_sum: [u32; 3] = [0, 0, 0];
            let mut area_count: u32 = 0;

            for xs in x_lb..=x_rb {
                for ys in y_tb..=y_bb {
                    let src_pixel = src_buffer.get_pixel(xs, ys);
                    color_sum[0] += src_pixel[0] as u32;
                    color_sum[1] += src_pixel[1] as u32;
                    color_sum[2] += src_pixel[2] as u32;
                    area_count += 1;
                }
            }
            let mean = Rgb::<u8>([
                (color_sum[0] / area_count) as u8,
                (color_sum[1] / area_count) as u8,
                (color_sum[2] / area_count) as u8,
            ]);

            for xs in x_lb..=x_rb {
                for ys in y_tb..=y_bb {
                    *buffer.get_pixel_mut(xs, ys) = mean;
                }
            }
        }
    }

    buffer
}
