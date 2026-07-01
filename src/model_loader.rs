// src/model_loader.rs

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct Model{
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }
    
    pub fn load_model<P: AsRef<Path>>(&mut self, path: P) {
        let file = File::open(path)
            .expect("Couldn't open file");

        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result
                .expect("Error reading line");

            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("v "){
                let coords: Vec<f32> = trimmed
                    .split_whitespace()
                    .skip(1)
                    .map(|s| s.parse::<f32>().unwrap())
                    .collect();
                self.vertices.push([coords[0], coords[1], coords[2]]);
            } else if trimmed.starts_with("f ") {
                let mut face_indices = [0; 3];
                let chunks = trimmed.split_whitespace().skip(1);
                for (i, chunk) in chunks.enumerate() {
                    if let Some(first_part) = chunk.split('/').next() {
                        let vertex_index = first_part
                            .parse::<usize>()
                            .expect("Failed to parse face index") - 1;
                        face_indices[i] = vertex_index;
                    }
                }
                self.faces.push(face_indices);
            }
        }
    }
}
