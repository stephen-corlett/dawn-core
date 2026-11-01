//! Bakes generated art into the game's assets. See forest.sh for how it's used.
//!
//! art-bake recompress <image> <out jpg>
//! art-bake solid <red> <green> <blue> <out jpg>
//! art-bake leaves <leaf png> <tiling image> <tiles> <detail strength> <rim strength> <out png>
//! art-bake triplanar <in gltf> <in bin> <out gltf> <out bin> <tiling image> <meters per tile>
//!     <size> <color uri> <out image>

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::imageops::{FilterType, blur, resize};
use image::{GrayImage, ImageEncoder, Luma, RgbImage};
use serde_json::{Value, json};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[0] {
        "recompress" => write_jpg(&image::open(args[1]).unwrap().to_rgb8(), args[2]),
        "solid" => {
            let v: Vec<u8> = args[1..4].iter().map(|a| a.parse().unwrap()).collect();
            let image = RgbImage::from_pixel(16, 16, image::Rgb([v[0], v[1], v[2]]));
            write_jpg(&image, args[4]);
        }
        "leaves" => leaves(&args[1..]),
        "triplanar" => triplanar(&args[1..]),
        other => panic!("unknown command {other}"),
    }
}

/// Projects a tiling texture onto a model from three directions, in 3D so that nothing joins at
/// UV edges, and bakes that into a texture of its own. The model's UVs are rescaled to fill it.
/// triplanar <in gltf> <in bin> <out gltf> <out bin> <tiling image> <meters per tile> <size>
/// <color uri> <out image>
fn triplanar(args: &[&str]) {
    let [gltf_in, bin_in, gltf_out, bin_out, stone, meters, size, uri, out] = args else {
        panic!("triplanar needs 9 arguments");
    };
    let meters: f32 = meters.parse().unwrap();
    let size: usize = size.parse().unwrap();
    let stone = image::open(stone).unwrap().to_rgb8();
    let (stone_w, stone_h) = stone.dimensions();
    let sample = |u: f32, v: f32| -> [f32; 3] {
        let x = u.rem_euclid(1.0) * stone_w as f32;
        let y = v.rem_euclid(1.0) * stone_h as f32;
        let (x0, y0) = (x as u32 % stone_w, y as u32 % stone_h);
        let (x1, y1) = ((x0 + 1) % stone_w, (y0 + 1) % stone_h);
        let (fx, fy) = (x.fract(), y.fract());
        let p = |a: u32, b: u32, c: usize| stone.get_pixel(a, b).0[c] as f32;
        [0, 1, 2].map(|c| {
            (p(x0, y0, c) * (1.0 - fx) + p(x1, y0, c) * fx) * (1.0 - fy)
                + (p(x0, y1, c) * (1.0 - fx) + p(x1, y1, c) * fx) * fy
        })
    };

    let mut gltf: Value = serde_json::from_str(&fs::read_to_string(gltf_in).unwrap()).unwrap();
    let mut bin = fs::read(bin_in).unwrap();
    let scale = gltf["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n.get("mesh").is_some())
        .filter_map(|n| n["scale"][0].as_f64())
        .fold(0.0f64, f64::max) as f32;
    let scale = if scale == 0.0 { 1.0 } else { scale };

    let read = |gltf: &Value, bin: &[u8], index: u64, width: usize| -> (Vec<f32>, usize, usize) {
        let accessor = &gltf["accessors"][index as usize];
        let view = &gltf["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
        let offset = view["byteOffset"].as_u64().unwrap_or(0) as usize
            + accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
        let stride = view["byteStride"].as_u64().unwrap_or(0) as usize;
        let stride = if stride == 0 { width * 4 } else { stride };
        let count = accessor["count"].as_u64().unwrap() as usize;
        let mut values = Vec::with_capacity(count * width);
        for vertex in 0..count {
            for channel in 0..width {
                let at = offset + vertex * stride + channel * 4;
                values.push(f32::from_le_bytes(bin[at..at + 4].try_into().unwrap()));
            }
        }
        (values, offset, stride)
    };

    let primitives: Vec<Value> = gltf["meshes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|mesh| mesh["primitives"].as_array().unwrap().clone())
        .collect();

    // Find the part of the shared atlas this model really uses.
    let (mut low, mut high) = ([f32::MAX; 2], [f32::MIN; 2]);
    for primitive in &primitives {
        let (uvs, _, _) = read(&gltf, &bin, primitive["attributes"]["TEXCOORD_0"].as_u64().unwrap(), 2);
        for uv in uvs.chunks(2) {
            for c in 0..2 {
                low[c] = low[c].min(uv[c]);
                high[c] = high[c].max(uv[c]);
            }
        }
    }
    let margin = 8.0 / size as f32;
    let extent = (high[0] - low[0]).max(high[1] - low[1]) * (1.0 + 2.0 * margin);
    println!("uses {:.2} of the atlas", extent);
    let remap = |u: f32, c: usize| ((u - low[c]) / extent + margin).clamp(0.0, 1.0);

    let mut color = vec![[0.0f32; 3]; size * size];
    let mut filled = vec![false; size * size];
    for primitive in &primitives {
        let attributes = &primitive["attributes"];
        let (positions, _, _) = read(&gltf, &bin, attributes["POSITION"].as_u64().unwrap(), 3);
        let (normals, _, _) = read(&gltf, &bin, attributes["NORMAL"].as_u64().unwrap(), 3);
        let (uvs, _, _) = read(&gltf, &bin, attributes["TEXCOORD_0"].as_u64().unwrap(), 2);
        let accessor = &gltf["accessors"][primitive["indices"].as_u64().unwrap() as usize];
        let view = &gltf["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
        let offset = view["byteOffset"].as_u64().unwrap_or(0) as usize
            + accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
        let count = accessor["count"].as_u64().unwrap() as usize;
        let indices: Vec<usize> = match accessor["componentType"].as_u64().unwrap() {
            5123 => (0..count)
                .map(|i| u16::from_le_bytes([bin[offset + i * 2], bin[offset + i * 2 + 1]]) as usize)
                .collect(),
            _ => (0..count)
                .map(|i| u32::from_le_bytes(bin[offset + i * 4..offset + i * 4 + 4].try_into().unwrap()) as usize)
                .collect(),
        };
        for tri in indices.chunks(3) {
            let uv = |i: usize| [remap(uvs[tri[i] * 2], 0) * size as f32, remap(uvs[tri[i] * 2 + 1], 1) * size as f32];
            let (a, b, c) = (uv(0), uv(1), uv(2));
            let area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
            if area.abs() < 1e-6 {
                continue;
            }
            let x0 = a[0].min(b[0]).min(c[0]).floor().max(0.0) as usize;
            let y0 = a[1].min(b[1]).min(c[1]).floor().max(0.0) as usize;
            let x1 = (a[0].max(b[0]).max(c[0]).ceil() as usize).min(size - 1);
            let y1 = (a[1].max(b[1]).max(c[1]).ceil() as usize).min(size - 1);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                    let w1 = ((px - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (py - a[1])) / area;
                    let w2 = ((b[0] - a[0]) * (py - a[1]) - (px - a[0]) * (b[1] - a[1])) / area;
                    let w0 = 1.0 - w1 - w2;
                    if w0 < -0.01 || w1 < -0.01 || w2 < -0.01 {
                        continue;
                    }
                    let mix = |data: &[f32], c: usize| {
                        w0 * data[tri[0] * 3 + c] + w1 * data[tri[1] * 3 + c] + w2 * data[tri[2] * 3 + c]
                    };
                    let p = [0, 1, 2].map(|c| mix(&positions, c) * scale / meters);
                    let n = [0, 1, 2].map(|c| mix(&normals, c));
                    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                    let mut w = n.map(|v| (v / length).abs().powi(6));
                    let total = w[0] + w[1] + w[2];
                    w = w.map(|v| v / total);
                    let (sx, sy, sz) = (sample(p[2], p[1]), sample(p[0], p[2]), sample(p[0], p[1]));
                    color[y * size + x] =
                        [0, 1, 2].map(|c| sx[c] * w[0] + sy[c] * w[1] + sz[c] * w[2]);
                    filled[y * size + x] = true;
                }
            }
        }
    }

    // Spread color past each island's edge, so filtering never reaches empty pixels.
    for _ in 0..8 {
        let before = filled.clone();
        let colors = color.clone();
        for y in 0..size {
            for x in 0..size {
                if before[y * size + x] {
                    continue;
                }
                let around = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)];
                if let Some(&(nx, ny)) = around
                    .iter()
                    .find(|(nx, ny)| *nx < size && *ny < size && before[ny * size + nx])
                {
                    color[y * size + x] = colors[ny * size + nx];
                    filled[y * size + x] = true;
                }
            }
        }
    }
    let image = RgbImage::from_fn(size as u32, size as u32, |x, y| {
        image::Rgb(color[y as usize * size + x as usize].map(|v| v.clamp(0.0, 255.0) as u8))
    });
    write_jpg(&image, out);

    // Rewrite the model's UVs in place, once each, since primitives can share them.
    let mut done = std::collections::HashSet::new();
    for primitive in &primitives {
        let index = primitive["attributes"]["TEXCOORD_0"].as_u64().unwrap();
        if !done.insert(index) {
            continue;
        }
        let (uvs, offset, stride) = read(&gltf, &bin, index, 2);
        for (vertex, uv) in uvs.chunks(2).enumerate() {
            for channel in 0..2 {
                let at = offset + vertex * stride + channel * 4;
                bin[at..at + 4].copy_from_slice(&remap(uv[channel], channel).to_le_bytes());
            }
        }
    }
    gltf["images"][0] = json!({ "uri": uri, "name": uri });
    fs::write(gltf_out, serde_json::to_string_pretty(&gltf).unwrap()).unwrap();
    fs::write(bin_out, bin).unwrap();
}

fn luma(pixel: [u8; 3]) -> f32 {
    (0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32) / 255.0
}

/// The scan's brightness around 1.0, repeating `tiles` times across a `size` square.
fn detail(scan: &str, tiles: u32, size: u32) -> Vec<f32> {
    let scan = image::open(scan).unwrap().to_rgb8();
    let tile = size / tiles;
    let small = resize(&scan, tile, tile, FilterType::Triangle);
    let values: Vec<f32> = small.pixels().map(|p| luma(p.0)).collect();
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    (0..size * size)
        .map(|i| {
            let (x, y) = ((i % size) % tile, (i / size) % tile);
            values[(y * tile + x) as usize] / mean
        })
        .collect()
}

fn write_jpg(image: &RgbImage, path: &str) {
    let file = fs::File::create(path).unwrap();
    JpegEncoder::new_with_quality(file, 88)
        .encode_image(image)
        .unwrap();
}

fn leaves(args: &[&str]) {
    let [painted, scan_color, tiles, detail_strength, rim_strength, out] = args else {
        panic!("leaves needs 6 arguments");
    };
    let tiles: u32 = tiles.parse().unwrap();
    let detail_strength: f32 = detail_strength.parse().unwrap();
    let rim_strength: f32 = rim_strength.parse().unwrap();

    let mut image = image::open(painted).unwrap().to_rgba8();
    let size = image.width();
    let bright = detail(scan_color, tiles, size);

    // Blurred coverage is low near a leaf's edge and high in its middle, which gives each
    // leaf the soft lit rim that the kit's painted leaves already have.
    let coverage = GrayImage::from_fn(size, size, |x, y| Luma([image.get_pixel(x, y).0[3]]));
    let coverage = blur(&coverage, 22.0);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let [r, g, b, a] = pixel.0;
        if a == 0 {
            continue;
        }
        let i = (y * size + x) as usize;
        let inner = ((coverage.get_pixel(x, y).0[0] as f32 / 255.0 - 0.5) * 2.0).clamp(0.0, 1.0);
        let scale = (1.0 + detail_strength * (bright[i] - 1.0)) * (1.0 + rim_strength * (0.5 - inner));
        let shade = |c: u8| ((c as f32 * scale).clamp(0.0, 255.0) as u8) & 0xFC;
        pixel.0 = [shade(r), shade(g), shade(b), a];
    }
    let file = fs::File::create(out).unwrap();
    PngEncoder::new_with_quality(file, CompressionType::Best, PngFilter::Adaptive)
        .write_image(image.as_raw(), size, size, image::ExtendedColorType::Rgba8)
        .unwrap();
}
