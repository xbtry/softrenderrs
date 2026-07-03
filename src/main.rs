mod model_loader;
use crate::model_loader::Model;
use rand::prelude::*;
use image::{ImageBuffer, Rgb};
use std::mem;
use std::thread;
use std::time::Instant;

#[derive(Debug,Clone,Copy)]
enum Color{
    // Primary & Secondary Colors
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    
    // Monochrome Palette
    White,
    LightGray,
    Gray,
    DarkGray,
    Black,
    
    // Extended Palette
    Orange,
    Brown,
    Purple,
    Pink,
    Teal,
    
    // Escape hatch for arbitrary colors
    Custom(u8, u8, u8),
}

impl Color{
    fn rgb(self) -> [u8; 3] {
        match self {
            // Primaries & Secondaries
            Color::Red     => [255, 0, 0],
            Color::Green   => [0, 255, 0],
            Color::Blue    => [0, 0, 255],
            Color::Yellow  => [255, 255, 0],
            Color::Cyan    => [0, 255, 255],
            Color::Magenta => [255, 0, 255],
            
            // Monochrome
            Color::White     => [255, 255, 255],
            Color::LightGray => [200, 200, 200],
            Color::Gray      => [128, 128, 128],
            Color::DarkGray  => [64, 64, 64],
            Color::Black     => [0, 0, 0],
            
            // Extended
            Color::Orange => [255, 165, 0],
            Color::Brown  => [139, 69, 19],
            Color::Purple => [128, 0, 128],
            Color::Pink   => [255, 192, 203],
            Color::Teal   => [0, 128, 128],
            
            Color::Custom(r, g, b) => [r, g, b],
        }
    }

    fn darker(self) -> Color {
        let [r,g,b] = self.rgb();
        Color::Custom(r/2, g/2, b/2)
    }

    fn lighter(self) -> Color {
        let[r,g,b] = self.rgb();
        Color::Custom(
            r.saturating_add(50),
            g.saturating_add(50),
            b.saturating_add(50),
        )
    }
}

struct Image{
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Image{
    fn new(width: u32, height: u32) -> Self{
        let total_bytes = (width * height * 3) as usize;
        Image{
            width,
            height,
            data: vec![0; total_bytes],
        }
    }

    
    fn save(&self, filename: &str){
        let mut img = ImageBuffer::new(self.width, self.height);

        for(x,y,pixel) in img.enumerate_pixels_mut(){

            let index = ((y * self.width + x) * 3) as usize;

            if index + 2 < self.data.len(){
                let r = self.data[index]; 
                let g = self.data[index+1];
                let b = self.data[index+2];
                *pixel = Rgb([r,g,b]);
            }
            else {
                *pixel = Rgb([0, 0, 0]);
            }
        }

        // 4. Write the buffer data to a file (format is inferred from extension)
        img.save(filename)
            .expect("Failed to write image data to disk");

        println!("Image successfully written: {}",filename);
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = ((y * self.width + x) * 3) as usize;

            let rgb_bytes = color.rgb();

            self.data[index..index + 3].copy_from_slice(&rgb_bytes);
        }
    }

    fn set_background(&mut self, color: Color) {
        let rgb_bytes = color.rgb();

        for chunk in self.data.chunks_exact_mut(3) {
            chunk.copy_from_slice(&rgb_bytes);
        }
    }

    fn line(&mut self, p0_x: u32, p0_y: u32, p1_x: u32, p1_y: u32, color: Color) {
        // Cast coordinates to floats upfront to handle negative directions safely
        let mut x0 = p0_x as f32;
        let mut y0 = p0_y as f32;
        let mut x1 = p1_x as f32;
        let mut y1 = p1_y as f32;
        
        let steep = (x0-x1).abs() < (y0 - y1).abs();

        if steep {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }

        let start_x = x0 as i32;
        let end_x = x1 as i32;

        if start_x == end_x {
            if steep {
                self.set_pixel(y0 as u32, start_x as u32, color);
            } else {
                self.set_pixel(start_x as u32, y0 as u32, color);
            }
            return;
        }

        for x_int in start_x..=end_x {
            let x = x_int as f32;
            let t = (x - x0) / (x1 - x0);
            let y = (y0 + t * (y1 - y0)).round() as u32;

            if steep {
                self.set_pixel(y, x_int as u32, color);
            } else {
                self.set_pixel(x_int as u32, y, color);
            }
        }
    }

    fn render_wireframe(&mut self, model: &Model) {
        let w = self.width as f32;
        let h = self.height as f32;

        for face in &model.faces {
            let idx0 = face[0];
            let idx1 = face[1];
            let idx2 = face[2];

            let v0 = model.vertices[idx0];
            let v1 = model.vertices[idx1];
            let v2 = model.vertices[idx2];

            let x0 = ((v0[0] + 1.0) * w / 2.0) as u32;
            let y0 = (h - ((v0[1] + 1.0)) * h / 2.0) as u32;

            let x1 = ((v1[0] + 1.0) * w / 2.0) as u32;
            let y1 = (h - ((v1[1] + 1.0)) * h / 2.0) as u32;

            let x2 = ((v2[0] + 1.0) * w / 2.0) as u32;
            let y2 = (h - ((v2[1] + 1.0)) * h / 2.0) as u32;

            self.line(x0, y0, x1, y1, Color::White);
            self.line(x1, y1, x2, y2, Color::White);
            self.line(x2, y2, x0, y0, Color::White);
        }
    }
    
    fn fill_bottom_triangle(&mut self, v0_x: u32, v0_y: u32, v1_x: u32, v1_y: u32, v2_x: u32, v2_y: u32, color: Color) {
        if v1_y == v0_y { return; }

        let invslope1_bottom = (v1_x as f32 - v0_x as f32) / (v1_y as f32 - v0_y as f32);
        let invslope2_bottom = (v2_x as f32 - v0_x as f32) / (v1_y as f32 - v0_y as f32);

        let mut curx1 = v0_x as f32;
        let mut curx2 = v0_x as f32;

        for i in v0_y..=v1_y {
            let curx1_u = curx1.round() as u32;
            let curx2_u = curx2.round() as u32;
            
            self.line(curx1_u, i, curx2_u, i, color);
            curx1 += invslope1_bottom;
            curx2 += invslope2_bottom;
        }
    }

    fn fill_top_triangle(&mut self, v0_x: u32, v0_y: u32, v1_x: u32, v1_y: u32, v2_x: u32, v2_y: u32, color: Color) {
        if v2_y == v0_y || v2_y == v1_y { return; }

        let invslope1_top = (v2_x as f32 - v0_x as f32) / (v2_y as f32 - v0_y as f32);
        let invslope2_top = (v2_x as f32 - v1_x as f32) / (v2_y as f32 - v1_y as f32);

        let mut curx1 = v0_x as f32;
        let mut curx2 = v1_x as f32;

        for i in v1_y..=v2_y {
            let curx1_u = curx1.round() as u32;
            let curx2_u = curx2.round() as u32;

            self.line(curx1_u, i, curx2_u, i, color);
            curx1 += invslope1_top;
            curx2 += invslope2_top;
        }
    }

    fn triangle(&mut self, v0_x: u32, v0_y: u32, v1_x: u32, v1_y: u32, v2_x: u32, v2_y: u32, color: Color){
        let mut vertices = [(v0_x, v0_y), (v1_x, v1_y), (v2_x, v2_y)];
        vertices.sort_by_key(|v| v.1);

        let (v0_x, v0_y) = vertices[0]; 
        let (v1_x, v1_y) = vertices[1];
        let (v2_x, v2_y) = vertices[2];

        if v0_y == v2_y {
            self.line(v0_x, v0_y, v1_x, v1_y, color);
            self.line(v1_x, v1_y, v2_x, v2_y, color);
            return;
        }

        let t = (v1_y as f32 - v0_y as f32) / (v2_y as f32 - v0_y as f32);
        let v3_x = (v0_x as f32 + t * (v2_x as f32 - v0_x as f32)).round() as u32;
        let v3_y = v1_y;

        self.fill_bottom_triangle(v0_x, v0_y, v1_x, v1_y, v3_x, v3_y, color);
        self.fill_top_triangle(v1_x, v1_y, v3_x, v3_y, v2_x, v2_y, color);
    }

    // Helper function for barycentric area weights
    fn signed_triangle_area(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> f32 {
        0.5 * ((by - ay) * (bx + ax) + (cy - by) * (cx + bx) + (ay - cy) * (ax + cx))
    }

    fn triangle_parallel(&mut self, v0_x: u32, v0_y: u32, v1_x: u32, v1_y: u32, v2_x: u32, v2_y: u32, color: Color) {
        // Cast everything to f32 up front for math
        let ax = v0_x as f32; let ay = v0_y as f32;
        let bx = v1_x as f32; let by = v1_y as f32;
        let cx = v2_x as f32; let cy = v2_y as f32;

        // 1. Compute Bounding Box
        let boundingbox_minx = v0_x.min(v1_x).min(v2_x).max(0) as usize;
        let boundingbox_miny = v0_y.min(v1_y).min(v2_y).max(0) as usize;
        let boundingbox_maxx = v0_x.max(v1_x).max(v2_x).min(self.width - 1) as usize;
        let boundingbox_maxy = v0_y.max(v1_y).max(v2_y).min(self.height - 1) as usize;
        
        // 2. Compute total area of the triangle
        let total_area = Self::signed_triangle_area(ax, ay, bx, by, cx, cy);
        if total_area.abs() < f32::EPSILON { return; } // Avoid division by zero for degenerate triangles

        let core_count = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        let total_rows = (boundingbox_maxy - boundingbox_miny) + 1;
        let rows_per_core = (total_rows + core_count - 1) / core_count;
        
        let width = self.width as usize;
        let pixels = &mut self.data;
        let rgb_bytes = color.rgb();
        
        // 3. Parallel Fork-Join Scope
        thread::scope(|s| {
            let block_size = rows_per_core * width * 3;
            
            for (i, block) in pixels.chunks_mut(block_size).enumerate() {
                s.spawn(move || {
                    let start_y = i * rows_per_core;
                    
                    for local_y in 0..rows_per_core {
                        let global_y = start_y + local_y;
                        
                        // Only process rows that fall inside our vertical bounding box
                        if global_y >= boundingbox_miny && global_y <= boundingbox_maxy {
                            let row_start_bytes = local_y * width * 3;
                            
                            for x in boundingbox_minx..=boundingbox_maxx {
                                let fx = x as f32;
                                let fy = global_y as f32;
                                
                                // Calculate barycentric coordinates using your signed area math
                                let alpha = Self::signed_triangle_area(fx, fy, bx, by, cx, cy) / total_area;
                                let beta  = Self::signed_triangle_area(fx, fy, cx, cy, ax, ay) / total_area;
                                let gamma = Self::signed_triangle_area(fx, fy, ax, ay, bx, by) / total_area;
                                
                                // If any coordinate is negative, the pixel is outside the triangle
                                if alpha < 0.0 || beta < 0.0 || gamma < 0.0 {
                                    continue;
                                }
                                
                                // Pixel is inside! Find the specific starting byte index
                                let pixel_start_idx = row_start_bytes + (x * 3);
                                
                                if pixel_start_idx + 2 < block.len() {
                                    block[pixel_start_idx..pixel_start_idx + 3].copy_from_slice(&rgb_bytes);
                                }
                            }
                        }
                    }
                });
            }
        });
    }
}


fn main() {
    const WIDTH: u32 = 2560;
    const HEIGHT: u32 = 1440;
    
    const TRIANGLE_COUNT: usize = 20000;

    let mut triangles = Vec::with_capacity(TRIANGLE_COUNT);

    for _ in 0..TRIANGLE_COUNT {
        let x0 = rand::random_range(0..WIDTH);
        let y0 = rand::random_range(0..HEIGHT);
        let x1 = rand::random_range(0..WIDTH);
        let y1 = rand::random_range(0..HEIGHT);
        let x2 = rand::random_range(0..WIDTH);
        let y2 = rand::random_range(0..HEIGHT);
        
        // Pick a random color variant
        let color = match rand::random_range(0..5) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Yellow,
            _ => Color::Cyan,
        };
        
        triangles.push((x0, y0, x1, y1, x2, y2, color));
    }
    let mut img = Image::new(WIDTH, HEIGHT);

    println!("Triangle count: {}", TRIANGLE_COUNT);

    println!("Parallel rendering starting..");
    let start_parallel = Instant::now();
    for t in &triangles {
        img.triangle_parallel(t.0, t.1, t.2, t.3, t.4, t.5, t.6);
    }
    let duration = start_parallel.elapsed();
    println!("Execution time: {:.2?}", duration);

    println!("Normal rendering starting..");
    let start_normal = Instant::now();
    for t in &triangles {
        img.triangle(t.0, t.1, t.2, t.3, t.4, t.5, t.6);
    }
    let duration_normal = start_normal.elapsed();
    println!("Execution time: {:.2?}", duration_normal);


//    img.save("output.png");
}
