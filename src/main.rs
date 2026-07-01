use image::{ImageBuffer, Rgb};
use std::mem;
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
}


fn main() {
    let mut img = Image::new(2560,1440);
    img.set_background(Color::Pink);
    img.line(50,50,500,500,Color::Black);
    img.save("output.png");
}
