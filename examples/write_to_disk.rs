use std::fs::File;
use std::env;

use ultimate_data_arc::ParseError;

fn main() {
    if let Some(file_name) = env::args().collect::<Vec<String>>().get(1) {
        if let Ok(file) = File::open(file_name) {
            match ultimate_data_arc::parse(file) {
                Ok(_) => {
                    println!("Do something cool with the data");
                }
                Err(ParseError::NotDataArc) => {
                    eprintln!("The file is not a valid data.arc file. (magic number was not detected)");
                }
                Err(ParseError::InternalError(err)) => {
                    eprintln!("Internal error, please report the entire error as a bug:\n\n{:?}", err);
                }
            }
        } else {
            eprintln!("File does not exist: {}", file_name);
        }
    } else {
        println!("Example usage: cargo run --example write_to_disk data.arc")
    }
}
