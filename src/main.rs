use image::{ImageBuffer, Rgb};

#[derive(Debug,Clone,Copy)]
enum Color{
    Red,
    Green,
    Blue,
    White,
    Black,
    Custom(u8,u8,u8),
}

impl Color{
    fn rgb(self) -> [u8; 3] {
        match self {
            Color::Red => [255,0,0],
            Color::Green => [0, 255, 0],
            Color::Blue => [0, 0, 255],
            Color::White => [255, 255, 255],
            Color::Black => [0, 0, 0],
            Color::Custom(r, g, b) => [r, g, b],
        }
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
}


fn main() {
    let mut img = Image::new(2,2);
    img.set_pixel(0,0,Color::Red);
    img.set_pixel(1,0,Color::Blue);
    img.set_pixel(0,1, Color::Green);
    img.set_pixel(1,1, Color::White);
    img.save("output.png");
}
