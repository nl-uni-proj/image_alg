use super::im::{self, RgbF};
use std::path::PathBuf;

pub fn run(target: &PathBuf) {
    if !target.exists() {
        eprintln!("path was not found `{}`", target.to_string_lossy());
        return;
    }
    if target.is_file() {
        analyze_image(target);
    }
    if target.is_dir() {
        let read_dir = std::fs::read_dir(target).expect("read dir");
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "png" {
                analyze_image(&path);
            }
        }
    }
}

fn analyze_image(path: &PathBuf) {
    let name = path
        .file_stem()
        .expect("image filename")
        .to_str()
        .expect("utf8 filename");

    let results_path = PathBuf::from("image_process_results");
    if !results_path.exists() {
        std::fs::create_dir(&results_path).expect("dir created");
    }
    let image_dir = results_path.join(name);
    if !image_dir.exists() {
        std::fs::create_dir(&image_dir).expect("dir created");
    }

    let image = im::image_open(&path);
    image_into_black_white(
        image.clone(),
        &image_dir.join(format!("{name}_black_white.png")),
    );
    image_into_grayscale(
        image,
        &image_dir.join(format!("{name}_grayscale.png")),
        &image_dir.join(format!("{name}_horizontal.png")),
        &image_dir.join(format!("{name}_vertical.png")),
        &image_dir.join(format!("{name}_bounds.png")),
    );
}

fn image_into_black_white(image: image::DynamicImage, save_path: &PathBuf) {
    let mut buffer = image.into_rgb8();

    for (_, _, pixel) in buffer.enumerate_pixels_mut() {
        let sum: u32 = pixel.0.iter().map(|&c| c as u32).sum();
        let mid: u32 = (255 * 3) / 2;
        *pixel = if sum >= mid {
            im::COLOR_WHITE
        } else {
            im::COLOR_BLACK
        };
    }

    im::image_buffer_save_png(buffer, save_path);
}

fn image_into_grayscale(
    image: image::DynamicImage,
    g_path: &PathBuf,
    h_path: &PathBuf,
    v_path: &PathBuf,
    m_path: &PathBuf,
) {
    let mut buffer = image.into_rgb8();

    for (_, _, pixel) in buffer.enumerate_pixels_mut() {
        // grayscale weights taken from: https://en.wikipedia.org/wiki/Grayscale
        let color = RgbF::from_u8(*pixel);
        let scale = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
        *pixel = RgbF::new(scale, scale, scale).into_u8();
    }

    // accumulate h and v projections from grayscale values
    let mut horizontal = Vec::new();
    let mut vertical = Vec::new();
    horizontal.resize_with(buffer.height() as usize, || 0.0);
    vertical.resize_with(buffer.width() as usize, || 0.0);
    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
        let scale = RgbF::from_u8(*pixel).r;
        horizontal[y as usize] += scale;
        vertical[x as usize] += scale;
    }

    // smooth and compute local minimas
    const SMOOTH_FACTOR: usize = 4;
    let h_reduced: Vec<f32> = horizontal
        .chunks(SMOOTH_FACTOR)
        .map(|chunk| chunk.iter().sum::<f32>() / (chunk.len() as f32))
        .collect();
    let v_reduced: Vec<f32> = vertical
        .chunks(SMOOTH_FACTOR)
        .map(|chunk| chunk.iter().sum::<f32>() / (chunk.len() as f32))
        .collect();
    let v_minimas = find_local_minimas(&v_reduced, 3);
    let h_minimas = find_local_minimas(&h_reduced, 3);

    // render h graph and minimas
    let mut h_buffer = buffer.clone();
    for (x, y, pixel) in h_buffer.enumerate_pixels_mut() {
        let idx = y as usize / SMOOTH_FACTOR;
        if h_minimas.iter().cloned().find(|&i| i == idx).is_some() {
            *pixel = im::COLOR_GREEN;
            continue;
        }

        if x < h_reduced[idx] as u32 {
            *pixel = im::COLOR_WHITE;
        } else if x <= h_reduced[idx] as u32 {
            *pixel = im::COLOR_RED;
        }
    }

    // render v graph and minimas
    let mut v_buffer = buffer.clone();
    for (x, y, pixel) in v_buffer.enumerate_pixels_mut() {
        let idx = x as usize / SMOOTH_FACTOR;
        if v_minimas.iter().cloned().find(|&i| i == idx).is_some() {
            *pixel = im::COLOR_GREEN;
            continue;
        }

        if buffer.height() - y < v_reduced[idx] as u32 {
            *pixel = im::COLOR_WHITE;
        } else if buffer.height() - y <= v_reduced[idx] as u32 {
            *pixel = im::COLOR_RED;
        }
    }

    // render bounds and minimas
    let mut m_buffer = buffer.clone();
    for (x, y, pixel) in m_buffer.enumerate_pixels_mut() {
        let h_idx = y as usize / SMOOTH_FACTOR;
        if h_minimas.iter().cloned().find(|&i| i == h_idx).is_some() {
            *pixel = im::COLOR_GREEN;
            continue;
        }
        let v_idx = x as usize / SMOOTH_FACTOR;
        if v_minimas.iter().cloned().find(|&i| i == v_idx).is_some() {
            *pixel = im::COLOR_GREEN;
            continue;
        }
    }

    im::image_buffer_save_png(buffer, g_path);
    im::image_buffer_save_png(h_buffer, h_path);
    im::image_buffer_save_png(v_buffer, v_path);
    im::image_buffer_save_png(m_buffer, m_path);
}

fn find_local_minimas(data: &[f32], minima_count: usize) -> Vec<usize> {
    let mut minima_scores = Vec::<f32>::new();

    // total left + right growth minima scoring
    for (mut idx, value) in data[1..data.len() - 1].iter().cloned().enumerate() {
        idx += 1;

        let mut left_last = value;
        let mut left_growth = 0.0;
        for v in data[0..idx - 1].iter().rev().cloned() {
            let diff = v - left_last;
            if diff <= 0.0 {
                break;
            }
            left_last = v;
            left_growth += diff;
        }

        let mut right_last = value;
        let mut right_growth = 0.0;
        for v in data[idx + 1..].iter().cloned() {
            let diff = v - right_last;
            if diff <= 0.0 {
                break;
            }
            right_last = v;
            right_growth += diff;
        }

        let minima_scope = if left_growth == 0.0 || right_growth == 0.0 {
            0.0
        } else {
            left_growth + right_growth
        };
        minima_scores.push(minima_scope);
    }

    // try to find `minima_count` indices based on max score
    let mut minima_indices = Vec::new();
    for _ in 0..minima_count {
        if let Some((index, _)) = minima_scores
            .iter()
            .enumerate()
            .max_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap())
        {
            minima_indices.push(index + 1);
            minima_scores[index] = f32::MIN;
        } else {
            break;
        }
    }

    minima_indices
}
