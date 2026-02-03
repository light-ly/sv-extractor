use std::{fs::{self, File}, path::PathBuf};
use clap::Parser;

mod hdl_info;
mod sv_parse;

#[derive(Parser)]
struct Args {
    #[arg(short = 'i', long = "input")]
    input: String,
    #[arg(short = 'o', long = "output")]
    output: String,
}

fn write_to_chisel(path: &PathBuf, name: &str, contents: &str) {
    if let Err(e) = fs::write(path.join(format!("{}.scala", name)), contents) {
        eprintln!("Failed to write {} to chisel: {}", name.to_string(), e);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input = PathBuf::from(args.input);
    let output = PathBuf::from(args.output);

    let mut hdl_info = hdl_info::HdlInfo::new();

    if input.is_dir() {
        let files = fs::read_dir(input).unwrap();
        for file in files {
            let file = file.unwrap();
            let info = sv_parse::parse_file(&file.path()).unwrap();
            hdl_info.merge_info(&info);
        }
    } else {
        let info = sv_parse::parse_file(&input).unwrap();
        hdl_info.merge_info(&info);
    }

    // println!("parse result: {:#?}", hdl_info);
    if output.exists() {
        assert!(output.is_dir(), "Error: Output should be a dir");
    } else {
        fs::create_dir_all(&output)?;
    }

    let json_file = File::create(output.join("hdl_info.json"))?;
    serde_json::to_writer_pretty(json_file, &hdl_info)?;

    let chisel_path = output.join("chisel");
    fs::create_dir_all(&chisel_path)?;

    hdl_info.get_modules().iter().for_each(|m| {
        write_to_chisel(&chisel_path, &m.get_name(), &m.to_chisel());
    });

    Ok(())
}
