use std::{error, fs, io};
use std::io::BufRead;

/// Load a polygon list from a file
pub fn load_polygon_list(path: &str) -> Result<Vec<Vec<[f32; 2]>>, Box<dyn error::Error>> {
    let mut output = Vec::new();
    let mut current = Vec::new();
    let f = fs::File::open(path)?;
    for line in io::BufReader::new(f).lines() {
        let line = line?;
        let mut chunks = line.split_ascii_whitespace();
        if let Some(x) = chunks.next() {
            let x = x.parse::<f32>()?;
            let y = chunks.next().ok_or(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Invalid input file")))?.parse::<f32>()?;
            current.push([x, y]);
        } else {
            let mut next = Vec::new();
            std::mem::swap(&mut current, &mut next);
            if !next.is_empty() {
                output.push(next);
            }
        }
    }

    Ok(output)
}