mod extract_data;

use extract_data::*;
fn main() {
    let path = "C:/Users/krut_ni/RustroverProjects/AIStudioDataPreparation/test-data/AuthorController.cs";

    let fmt_result = extract_data(path);
    match fmt_result {
        Ok(data) => {println!("{}", data)},
        Err(e) => {println!("{}", e)}
    }
}
