use std::{fs, path::PathBuf};
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

fn main() {
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

    println!("parse result: {:#?}", hdl_info);
}
