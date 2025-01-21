use std::error::Error;
use std::fs;
use file_format::{FileFormat, Kind};
use std::fs::{exists, File};
use std::io::Read;
use std::process::Command;

const TO_MARKDOWN: &str = "markdown";

pub fn extract_data(file_path: &str) -> Result<String, Box<dyn Error>> {
    if let Err(_) = exists(file_path) {
        return Err(Box::from("File does not exist."));
    }

    let fmt = FileFormat::from_file(file_path)?;
    match fmt.kind() {
        Kind::Document => {
            match fmt {
                FileFormat::PortableDocumentFormat => {
                    convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN)
                },
                FileFormat::MicrosoftWordDocument => {
                    convert_with_pandoc(file_path, "docx", TO_MARKDOWN)
                },
                FileFormat::OfficeOpenXmlDocument => {
                    convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN)
                },
                _ => Ok(try_read_file(file_path)?),
            }
        }
        Kind::Ebook => {
            match fmt {
                _ => Ok(format!("TODO: '{:?}' of kind: '{:?}'", fmt, fmt.kind())),
            }
        },
        Kind::Image => {
            match fmt {
                _ => Ok(format!("TODO: {:?} of kind: {:?}", fmt, fmt.kind())),
            }
        },
        Kind::Other => {
            let content = try_read_file(file_path)?;
            Ok(content)
        },
        Kind::Presentation => {
            match fmt {
                FileFormat::OfficeOpenXmlPresentation => { convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN) },
                _ => Ok(try_read_file(file_path)?),
            }
        },
        Kind::Spreadsheet => {
            match fmt {
                _ => Ok(format!("TODO: {:?} of kind: {:?}", fmt, fmt.kind())),
            }
        },
        _ => Ok(try_read_file(file_path)?),
    }
}

fn convert_with_pandoc(file_path: &str, from: &str, to: &str) -> Result<String, Box<dyn Error>> {
    // pandoc [-f FORMAT] [-t FORMAT]
    let cmd = Command::new("pandoc")
        .arg(file_path)
        .arg("-f")
        .arg(from.to_lowercase())
        .arg("-t")
        .arg(to)
        .output()?;

    if cmd.status.success() {
        let content = String::from_utf8(cmd.stdout)?;
        println!("{}", content);
        Ok(content)
    } else {
        let stderr = String::from_utf8_lossy(&cmd.stderr);
        Err(Box::from(format!("Pandoc-Konvertierung von '{}' nach '{}' fehlgeschlagen: {}", from, to, stderr)))
    }
}

fn try_read_file(file_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    let result = file.read_to_string(&mut contents);

    match result {
        Ok(_) => Ok(contents),
        Err(e) => Err(Box::from(format!("{}", e))),
    }
}