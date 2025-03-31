mod stream_data;

use std::error::Error;
use clap::Parser;
use futures::StreamExt;
use stream_data::{stream_data, Metadata};

#[derive(Parser)]
#[command(
    version = "0.1.0",
    about = "Isolated program to test data extraction for AIStudio",
)]
struct Args {
    #[arg(
        long,
        short = 'p',
        value_name = "PATH",
        help = "The path to the file containing the data to be extracted.",
        required = true,
    )]
    path: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();
    let path_str = args.path.to_str().unwrap();

    let mut chunk_stream = stream_data(path_str).await?;

    while let Some(chunk_result) = chunk_stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let meta_info = match &chunk.metadata {
                    Metadata::Text { line_number } =>
                        format!("Line {}", line_number),
                    Metadata::Pdf { page_number } =>
                        format!("Page {}", page_number),
                    Metadata::Spreadsheet { sheet_name, row_number } =>
                        format!("Sheet '{}' Row {}", sheet_name, row_number),
                    Metadata::Document => "Full document".to_string(),
                    Metadata::Image => "Image data".to_string(),
                };
                println!("=== {} ===\n{}\n", meta_info, chunk.content);
            }
            Err(e) => eprintln!("Chunk error: {}", e),
        }
    }

    Ok(())
}