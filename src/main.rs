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
        if total_area < 1.0 { return; }

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
    let w = WIDTH as f32;
    let h = HEIGHT as f32;
    let mut model = Model::new();
    model.load_model("assets/diablo3_pose.obj");
    let mut img = Image::new(WIDTH,HEIGHT);
    let mut triangles = Vec::new();

    for face in &model.faces {
        let idx0 = face[0];
        let idx1 = face[1];
        let idx2 = face[2];

        let v0 = model.vertices[idx0];
        let v1 = model.vertices[idx1];
        let v2 = model.vertices[idx2];

        // Convert raw model floats [-1.0, 1.0] to screen space pixel coordinates (u32)
        let ax = ((v0[0] + 1.0) * w / 2.0) as u32;
        let ay = (h - (v0[1] + 1.0) * h / 2.0) as u32;

        let bx = ((v1[0] + 1.0) * w / 2.0) as u32;
        let by = (h - (v1[1] + 1.0) * h / 2.0) as u32;

        let cx = ((v2[0] + 1.0) * w / 2.0) as u32;
        let cy = (h - (v2[1] + 1.0) * h / 2.0) as u32;

        let color = match rand::random_range(0..5) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Yellow,
            _ => Color::Cyan,
        }; 

        triangles.push((ax, ay, bx, by, cx, cy, color));
    }

    img.parallel_triangle(&triangles);
    img.save("output.png");
}
