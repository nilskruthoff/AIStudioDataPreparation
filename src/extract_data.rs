use base64::{engine::general_purpose, Engine as _};
use calamine::{open_workbook_auto, DataType, Reader};
use file_format::{FileFormat, Kind};
use pdf_extract::extract_text_from_mem;
use std::error::Error;
use std::fs::{exists, read, File};
use std::io::Read;
use std::process::Command;

const TO_MARKDOWN: &str = "markdown";
const DOCX: &str = "docx";
const ODT: &str = "odt";

pub fn extract_data(file_path: &str) -> Result<String, Box<dyn Error>> {
    if let Err(_) = exists(file_path) {
        return Err(Box::from("File does not exist."));
    }

    let fmt = FileFormat::from_file(file_path)?;
    let ext = file_path.split('.').last().unwrap();
    println!("{:?}", ext);

    match ext {
        DOCX => return convert_with_pandoc(file_path, DOCX, TO_MARKDOWN),
        ODT => return convert_with_pandoc(file_path, ODT, TO_MARKDOWN),
        "xlsx" | "ods" | "xls" | "xlsm"
        | "xlsb" | "xla" | "xlam" => return read_spreadsheet_as_csv(file_path),
        _ => {}
    }

    println!("Kind {:?}, Format {:?}, Media Type {:?}", fmt.kind(), fmt, fmt.media_type());
    match fmt.kind() {
        Kind::Document => {
            match fmt {
                FileFormat::PortableDocumentFormat => {
                    read_pdf(file_path)
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
            match fmt {
                FileFormat::HypertextMarkupLanguage => {
                    convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN)
                },
                _ => Ok(try_read_file(file_path)?),
            }
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
                    Ok(read_spreadsheet_as_csv(file_path)?)
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
    let img_result = File::open(file_path);

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

fn read_spreadsheet_as_csv(file_path: &str) -> Result<String, Box<dyn Error>> {
    let mut workbook = open_workbook_auto(file_path)?;
    let sheet_names = workbook.sheet_names().to_owned();
    let mut csv_str = String::new();

    for sheet_name in &sheet_names {
        match workbook.worksheet_range(sheet_name) {
            Ok(range) => {
                csv_str.push_str(&format!("{}:\n", sheet_name));
                for row in range.rows() {
                    let row_str: Vec<String> = row.iter().map(|cell| {
                        if cell.is_empty() {
                            "".to_string()
                        } else if cell.is_string() {
                            cell.get_string().unwrap_or("").trim().to_string()
                        } else if cell.is_int() {
                            cell.get_int().unwrap_or(0).to_string()
                        } else if cell.is_float() {
                            cell.get_float().unwrap_or(0.0).to_string()
                        } else if cell.is_bool() {
                            cell.get_bool().unwrap_or(false).to_string()
                        } else if cell.is_datetime() {
                            if let Some(dt) = cell.get_datetime() {
                                if let Some(datetime) = dt.as_datetime() {
                                    datetime.format("%d.%m.%Y %H:%M:%S").to_string()
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string()
                            }
                        }
                        else {
                            "".to_string()
                        }
                    }).collect();
                    csv_str.push_str(&row_str.join(","));
                    csv_str.push('\n');
                }
            }
            Err(e) => {
                csv_str.push_str(&format!("Das Arbeitsblatt '{}' konnte nicht gelesen werden: {}\n", sheet_name, e));
            }
        }
    }
    Ok(csv_str)
}

fn read_pdf(file_path: &str) -> Result<String, Box<dyn Error>> {
    let bytes = read(file_path)?;
    let out = extract_text_from_mem(&bytes)?;
    Ok(out)
}