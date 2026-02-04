use std::{fs::{self, File}, path::PathBuf};
use clap::Parser;

mod hdl_info;
mod sv_parse;
mod converter;

use crate::converter::ChiselConverter;

#[derive(Parser)]
struct Args {
    #[arg(short = 'i', long = "input")]
    input: String,
    #[arg(short = 'o', long = "output")]
    output: String,
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

    let _chisel_str = ChiselConverter::builder().emit_chisel_string(&hdl_info);
    ChiselConverter::builder().emit_chisel(&chisel_path, &hdl_info);
    ChiselConverter::builder().split_bundle().emit_chisel(&chisel_path.join("split_bundle"), &hdl_info);

    Ok(())
}
