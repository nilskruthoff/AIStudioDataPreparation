﻿use std::path::Path;
use std::pin::Pin;
use async_stream::stream;
use base64::{engine::general_purpose, Engine as _};
use calamine::{open_workbook_auto, Reader};
use file_format::{FileFormat, Kind};
use futures::{Stream, StreamExt};
use pdfium_render::prelude::Pdfium;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct Chunk {
    pub content: String,
    pub metadata: Metadata,
}

#[derive(Debug)]
pub enum Metadata {
    Text { line_number: usize },
    Pdf { page_number: usize },
    Spreadsheet { sheet_name: String, row_number: usize },
    Document,
    Image,
}

const TO_MARKDOWN: &str = "markdown";
const DOCX: &str = "docx";
const ODT: &str = "odt";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type ChunkStream = Pin<Box<dyn Stream<Item = Result<Chunk>> + Send>>;

pub async fn stream_data(file_path: &str) -> Result<ChunkStream> {
    if !Path::new(file_path).exists() {
        return Err("File does not exist.".into());
    }

    let file_path_clone = file_path.to_owned();

    let fmt = tokio::task::spawn_blocking(move || {
        FileFormat::from_file(&file_path_clone)
    }).await??;

    let ext = file_path.split('.').last().unwrap_or("");

    let stream = match ext {
        DOCX | ODT => {
            let from = if ext == DOCX { "docx" } else { "odt" };
            convert_with_pandoc(file_path, from, TO_MARKDOWN).await?
        }
        "xlsx" | "ods" | "xls" | "xlsm" | "xlsb" | "xla" | "xlam" => {
            stream_spreadsheet_as_csv(file_path).await?
        }
        _ => match fmt.kind() {
            Kind::Document => match fmt {
                FileFormat::PortableDocumentFormat => read_pdf(file_path).await?,
                FileFormat::MicrosoftWordDocument => {
                    convert_with_pandoc(file_path, "docx", TO_MARKDOWN).await?
                }
                FileFormat::OfficeOpenXmlDocument => {
                    convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN).await?
                }
                _ => stream_text_file(file_path).await?,
            },
            Kind::Ebook => return Err("Ebooks not yet supported".into()),
            Kind::Image => chunk_image(file_path).await?,
            Kind::Other => match fmt {
                FileFormat::HypertextMarkupLanguage => {
                    convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN).await?
                }
                _ => stream_text_file(file_path).await?,
            },
            Kind::Presentation => match fmt {
                FileFormat::OfficeOpenXmlPresentation => {
                    convert_with_pandoc(file_path, fmt.extension(), TO_MARKDOWN).await?
                }
                _ => stream_text_file(file_path).await?,
            },
            Kind::Spreadsheet => stream_spreadsheet_as_csv(file_path).await?,
            _ => stream_text_file(file_path).await?,
        },
    };

    Ok(Box::pin(stream))
}

async fn stream_text_file(file_path: &str) -> Result<ChunkStream> {
    let file = tokio::fs::File::open(file_path).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut line_number = 0;

    let stream = stream! {
        while let Ok(Some(line)) = lines.next_line().await { // Korrektur hier
            line_number += 1;
            yield Ok(Chunk {
                content: line,
                metadata: Metadata::Text { line_number },
            });
        }
    };

    Ok(Box::pin(stream))
}

async fn read_pdf(file_path: &str) -> Result<ChunkStream> {
    let path = file_path.to_owned();
    let (tx, rx) = mpsc::channel(10);

    tokio::task::spawn_blocking(move || {
        let pdfium = Pdfium::default();
        let doc = match pdfium.load_pdf_from_file(&path, None) {
            Ok(d) => d,
            Err(e) => {
                let _ = tx.blocking_send(Err(e.into()));
                return;
            }
        };

        for (i, page) in doc.pages().iter().enumerate() {
            let content = match page.text().and_then(|t| Ok(t.all())) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.blocking_send(Err(e.into()));
                    continue;
                }
            };

            if tx.blocking_send(Ok(Chunk {
                content,
                metadata: Metadata::Pdf { page_number: i + 1 },
            })).is_err() {
                break;
            }
        }
    });

    Ok(Box::pin(ReceiverStream::new(rx)))
}

async fn stream_spreadsheet_as_csv(file_path: &str) -> Result<ChunkStream> {
    let path = file_path.to_owned();
    let (tx, rx) = mpsc::channel(10);

    tokio::task::spawn_blocking(move || {
        let mut workbook = match open_workbook_auto(&path) {
            Ok(w) => w,
            Err(e) => {
                let _ = tx.blocking_send(Err(e.into()));
                return;
            }
        };

        for sheet_name in workbook.sheet_names() {
            let range = match workbook.worksheet_range(&sheet_name) {
                Some(Ok(r)) => r,
                Some(Err(e)) => {
                    let _ = tx.blocking_send(Err(e.into()));
                    continue;
                }
                None => continue,
            };

            for (row_idx, row) in range.rows().enumerate() {
                let content = row.iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                if tx.blocking_send(Ok(Chunk {
                    content,
                    metadata: Metadata::Spreadsheet {
                        sheet_name: sheet_name.clone(),
                        row_number: row_idx + 1,
                    },
                })).is_err() {
                    return;
                }
            }
        }
    });

    Ok(Box::pin(ReceiverStream::new(rx)))
}

async fn convert_with_pandoc(
    file_path: &str,
    from: &str,
    to: &str,
) -> Result<ChunkStream> {
    let output = Command::new("pandoc")
        .arg(file_path)
        .args(&["-f", from, "-t", to])
        .output()
        .await?;

    let stream = stream! {
        if output.status.success() {
            match String::from_utf8(output.stdout.clone()) {
                Ok(content) => yield Ok(Chunk {
                    content,
                    metadata: Metadata::Document,
                }),
                Err(e) => yield Err(e.into()),
            }
        } else {
            yield Err(format!(
                "Pandoc error: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
    };

    Ok(Box::pin(stream))
}

async fn chunk_image(file_path: &str) -> Result<ChunkStream> {
    let data = tokio::fs::read(file_path).await?;
    let base64 = general_purpose::STANDARD.encode(&data);

    let stream = stream! {
        yield Ok(Chunk {
            content: base64,
            metadata: Metadata::Image,
        });
    };

    Ok(Box::pin(stream))
}