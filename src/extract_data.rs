use std::error::Error;
use file_format::{FileFormat, Kind};
use std::fs::exists;

pub fn extract_data(file_path: &str) -> Result<String, Box<dyn Error>> {
    if let Err(_) = exists(file_path) {
        return Err(Box::from("File does not exist."));
    }

    let fmt = FileFormat::from_file(file_path)?;
    let kind = fmt.kind();
    println!("kind: {:?}", &kind);
    match kind {
        Kind::Other => Ok(file_path.to_string()),
        _ => { Err(Box::from("File does not support kind.")) }
    }
}