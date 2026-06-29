use image::{ImageBuffer, Rgb};

fn save_image(width: u32, height: u32, data: &[u8], filename: &str){
    let mut img = ImageBuffer::new(width, height);

    for(x,y,pixel) in img.enumerate_pixels_mut(){

        let index = ((y * width + x) * 3) as usize;

        if index + 2 < data.len(){
            let r = data[index]; 
            let g = data[index+1];
            let b = data[index+2];
            *pixel = Rgb([r,g,b]);
        }
        else {
            *pixel = Rgb([0, 0, 0]);
        }
    }

    // 4. Write the buffer data to a file (format is inferred from extension)
    img.save(filename)
        .expect("Failed to write image data to disk");

    println!("Image successfully written: {}!",filename);
}

fn main() {
    let width = 2;
    let height = 2;
    let dummy_data: Vec<u8> = vec![
        255,0,0,
        255,255,0,
        255,255,255,
        0,255,0,
    ];
    save_image(width, height, &dummy_data, "output.png");
}
