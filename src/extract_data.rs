﻿use std::error::Error;
use std::io;
use file_format::{FileFormat, Kind};
use std::fs::{exists, File};
use std::io::Read;
use std::process::Command;
use base64::{engine::general_purpose, Engine as _};
use calamine::{open_workbook, Reader, Xlsx, XlsxError};

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
                FileFormat::JointPhotographicExpertsGroup |
                FileFormat::PortableNetworkGraphics |
                FileFormat::Webp |
                FileFormat::TagImageFileFormat |
                FileFormat::ScalableVectorGraphics |
                FileFormat::RadianceHdr |
                FileFormat::WindowsBitmap => {
                    Ok(read_img_as_base64(file_path)?)
                }
                _ => Ok(format!("Bilder vom Typen '{:?}' werden nicht unterstützt", fmt.kind())),
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
                FileFormat::OfficeOpenXmlSpreadsheet => {
                    Ok(read_xlsx_as_csv(file_path)?)
                },
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

fn read_img_as_base64(file_path: &str) -> Result<String, Box<dyn Error>> {
    let mut img_result = File::open(file_path);

    match img_result {
        Ok(mut img) => {
            let mut buff = Vec::new();
            img.read_to_end(&mut buff)?;

            let base64 = general_purpose::STANDARD.encode(&buff);
            Ok(base64)
        }
        Err(e) => Err(Box::from(format!("{}", e))),
    }
}

fn read_xlsx_as_csv(file_path: &str) -> io::Result<String> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)
        .map_err(|xlsx_error: XlsxError| io::Error::new(io::ErrorKind::Other, xlsx_error.to_string()))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let mut csv_str = String::new();

    for sheet_name in sheet_names {
        let mut sheet_str = String::new();
        let range = workbook
            .worksheet_range(&*sheet_name)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        for row in range.rows() {
            let row_str: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
            sheet_str.push_str(&row_str.join(",\t"));
            sheet_str.push('\n');
        }

        csv_str.push_str(&format!("{}:\n", sheet_name));
        csv_str.push_str(&format!("{}\n", &sheet_str));
    }

    Ok(csv_str)
}