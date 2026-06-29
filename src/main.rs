use image::{ImageBuffer, Rgb};

fn save_image(width: u32, height: u32, filename: &str){
    let mut img = ImageBuffer::new(width, height);

    for(x,y,pixel) in img.enumerate_pixels_mut(){
        let r = (x as f32 / width as f32 * 255.0) as u8;
        let g = (y as f32 / height as f32 * 255.0) as u8;
        let b = 150; // Keep the blue channel constant

        // Assign the color value to the active pixel
        *pixel = Rgb([r, g, b]);
    }

    // 4. Write the buffer data to a file (format is inferred from extension)
    img.save(filename)
        .expect("Failed to write image data to disk");

    println!("Image successfully written: {}!",filename);
}

fn main() {
    save_image(500,500,"gradient.png")
}
