mod model_loader;
use crate::model_loader::Model;
use image::{ImageBuffer, Rgb};
use std::mem;
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    LightGray,
    Gray,
    DarkGray,
    Black,
    Orange,
    Brown,
    Purple,
    Pink,
    Teal,
    Custom(u8, u8, u8),
}

impl Color {
    fn rgb(self) -> [u8; 3] {
        match self {
            Color::Red       => [255, 0, 0],
            Color::Green     => [0, 255, 0],
            Color::Blue      => [0, 0, 255],
            Color::Yellow    => [255, 255, 0],
            Color::Cyan      => [0, 255, 255],
            Color::Magenta   => [255, 0, 255],
            Color::White     => [255, 255, 255],
            Color::LightGray => [200, 200, 200],
            Color::Gray      => [128, 128, 128],
            Color::DarkGray  => [64, 64, 64],
            Color::Black     => [0, 0, 0],
            Color::Orange    => [255, 165, 0],
            Color::Brown     => [139, 69, 19],
            Color::Purple    => [128, 0, 128],
            Color::Pink      => [255, 192, 203],
            Color::Teal      => [0, 128, 128],
            Color::Custom(r, g, b) => [r, g, b],
        }
    }
}

struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Image {
    fn new(width: u32, height: u32) -> Self {
        let total_bytes = (width * height * 3) as usize;
        Image {
            width,
            height,
            data: vec![0; total_bytes],
        }
    }

    fn save(&self, filename: &str) {
        let mut img = ImageBuffer::new(self.width, self.height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let index = ((y * self.width + x) * 3) as usize;
            if index + 2 < self.data.len() {
                let r = self.data[index];
                let g = self.data[index + 1];
                let b = self.data[index + 2];
                *pixel = Rgb([r, g, b]);
            } else {
                *pixel = Rgb([0, 0, 0]);
            }
        }
        img.save(filename).expect("Failed to write image data to disk");
        println!("Image successfully written: {}", filename);
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = ((y * self.width + x) * 3) as usize;
            let rgb_bytes = color.rgb();
            self.data[index..index + 3].copy_from_slice(&rgb_bytes);
        }
    }

    fn signed_triangle_area(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> f32 {
        0.5 * ((by - ay) * (bx + ax) + (cy - by) * (cx + bx) + (ay - cy) * (ax + cx))
    }

    fn triangle_barycentric(&mut self, v0_x: u32, v0_y: u32, v1_x: u32, v1_y: u32, v2_x: u32, v2_y: u32, color: Color) {
        let ax = v0_x as f32; let ay = v0_y as f32;
        let bx = v1_x as f32; let by = v1_y as f32;
        let cx = v2_x as f32; let cy = v2_y as f32;

        let bbminx = v0_x.min(v1_x).min(v2_x).max(0);
        let bbminy = v0_y.min(v1_y).min(v2_y).max(0);
        let bbmaxx = v0_x.max(v1_x).max(v2_x).min(self.width - 1);
        let bbmaxy = v0_y.max(v1_y).max(v2_y).min(self.height - 1);

        let total_area = Self::signed_triangle_area(ax, ay, bx, by, cx, cy);
        if total_area.abs() < f32::EPSILON { return; }

        for y in bbminy..=bbmaxy {
            for x in bbminx..=bbmaxx {
                let fx = x as f32;
                let fy = y as f32;

                let alpha = Self::signed_triangle_area(fx, fy, bx, by, cx, cy) / total_area;
                let beta  = Self::signed_triangle_area(fx, fy, cx, cy, ax, ay) / total_area;
                let gamma = Self::signed_triangle_area(fx, fy, ax, ay, bx, by) / total_area;

                if alpha < 0.0 || beta < 0.0 || gamma < 0.0 { continue; }

                self.set_pixel(x, y, color);
            }
        }
    }
    
    fn parallel_triangle(&mut self, triangles: &[(u32, u32, u32, u32, u32, u32, Color)]) {
        let core_count = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        let chunk_size = (triangles.len() + core_count - 1) / core_count;
        let img_width = self.width;
        let img_height = self.height;

        let mut local_buffers = thread::scope(|s| {
            let mut handles = vec![];
            for chunk in triangles.chunks(chunk_size) {
                let handle = s.spawn(move || {
                    let mut local_img = Image::new(img_width, img_height);
                    for t in chunk {
                        local_img.triangle_barycentric(t.0, t.1, t.2, t.3, t.4, t.5, t.6);
                    }
                    local_img.data
                });
                handles.push(handle);
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u8>>>()
        });

        for local_data in local_buffers {
            for (dest, src) in self.data.iter_mut().zip(local_data.iter()) {
                if *src != 0 {
                    *dest = *src;
                }
            }
        }
    }
}

fn main() {
    const WIDTH: u32 = 2560;
    const HEIGHT: u32 = 1440;
    const TRIANGLE_COUNT: usize = 20000;

    let mut triangles = Vec::with_capacity(TRIANGLE_COUNT);
    let mut img_seq = Image::new(WIDTH, HEIGHT);
    let mut img_par = Image::new(WIDTH, HEIGHT);

    for _ in 0..TRIANGLE_COUNT {
        let x0 = rand::random_range(0..WIDTH);
        let y0 = rand::random_range(0..HEIGHT);
        let x1 = rand::random_range(0..WIDTH);
        let y1 = rand::random_range(0..HEIGHT);
        let x2 = rand::random_range(0..WIDTH);
        let y2 = rand::random_range(0..HEIGHT);

        let color = match rand::random_range(0..5) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Yellow,
            _ => Color::Cyan,
        };

        triangles.push((x0, y0, x1, y1, x2, y2, color));
    }

    println!("Total Triangle count: {}", TRIANGLE_COUNT);
    println!("--------------------------------------------------");

    let start_seq = Instant::now();
    for t in &triangles {
        img_seq.triangle_barycentric(t.0, t.1, t.2, t.3, t.4, t.5, t.6);
    }
    let duration_seq = start_seq.elapsed();
    println!("Normal execution total time:   {:.2?}", duration_seq);
    println!("--------------------------------------------------");

    let start_par = Instant::now();
    img_par.parallel_triangle(&triangles);
    let duration_par = start_par.elapsed();
    println!("Parallel execution total time: {:.2?}", duration_par);
    println!("--------------------------------------------------");

    img_par.save("output.png");
}
