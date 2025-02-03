mod extract_data;

use extract_data::*;
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

    let fmt_result = extract_data(path_str);
    match fmt_result {
        Ok(data) => {
            println!("{}", data)
        }
        Err(e) => {
            println!("{}", e)
        }
    }
}
