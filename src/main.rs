use std::{fs, path::PathBuf};
use clap::Parser;

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

    if input.is_dir() {
        let files = fs::read_dir(input).unwrap();
        for file in files {
            let file = file.unwrap();
            let module = sv_parse::parse_file(&file.path()).unwrap();
            println!("Module: {:#?}", module);
        }
    } else {
        let module = sv_parse::parse_file(&input).unwrap();
        println!("Module: {:#?}", module);
    }
}
