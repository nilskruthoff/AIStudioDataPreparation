mod stream_data;

use stream_data::*;
use clap::Parser;

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

fn main() {
    let args = Args::parse();
    let path_str = args.path.to_str().unwrap();

    match stream_data(path_str) {
        Err(e) => eprintln!("Extraction failed: {}", e),

        Ok(chunk_stream) => {
            for chunk_result in chunk_stream {
                match chunk_result {
                    Err(e) => eprintln!("Chunk error: {}", e),

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
                }
            }
        }
    }
}