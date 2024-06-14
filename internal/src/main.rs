use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <input_file>", args[0]);
        std::process::exit(1)
    }

    let path = &args[1];

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            println!("Error: {}", error);
            std::process::exit(1)
        }
    };

    let mut input_data = String::new();
    file.read_to_string(&mut input_data).unwrap();
    let input_data: serde_json::Value = serde_json::from_str(&input_data).unwrap();
    println!("\n---------- RUN: {} ----------", path);

    let output = internal::run(input_data);

    // output path with sub-directory creation
    let output_dir_name = "output";
    let output_path = ensure_output_path(path, output_dir_name);
    let file = File::create(output_path).expect("Error creating file");
    serde_json::to_writer_pretty(file, &output).expect("Error writing JSON");

    std::process::exit(0)
}

fn ensure_output_path(input_path: &str, output_dir_name: &str) -> String {
    let file_name = Path::new(input_path)
        .file_name()
        .expect("Error getting file name")
        .to_str()
        .expect("Error converting file name to string");
    let output_path = format!("{}/output_{}", output_dir_name, file_name);
    if let Some(parent_dir) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent_dir).expect("Error creating directories");
    }
    output_path
}
