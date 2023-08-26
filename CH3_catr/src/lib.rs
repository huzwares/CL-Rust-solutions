use clap::{Arg, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(e) => eprintln!("Failed to open {}: {}", filename, e),
            Ok(f) => {
                let mut counter = 0;
                for line in f.lines() {
                    let line = line?;
                    if config.number_lines {
                        counter += 1;
                        println!("{:6}\t{}", counter, line);
                    } else if config.number_nonblank_lines {
                        if !line.is_empty() {
                            counter += 1;
                            println!("{:6}\t{}", counter, line);
                        } else {
                            println!();
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("catr")
        .version("0.1.0")
        .author("huzwares <huzwares@skiff.com>")
        .about("Rust cat")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .required(false)
                .action(clap::ArgAction::Append)
                .value_parser(clap::value_parser!(String))
                .help("Input file(s)")
                .default_value("-"),
        )
        .arg(
            Arg::new("number_lines")
                .long("number")
                .short('n')
                .help("Number the output lines, starting at 1.")
                .conflicts_with("number_nonblank_lines")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("number_nonblank_lines")
                .long("number-nonblank")
                .short('b')
                .help("Number the non-blank output lines, starting at 1.")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("files")
        .unwrap()
        .map(|s| s.as_str().to_string())
        .collect();
    let number_lines = matches.get_one::<bool>("number_lines").unwrap().clone();
    let number_nonblank_lines = matches
        .get_one::<bool>("number_nonblank_lines")
        .unwrap()
        .clone();
    Ok(Config {
        files,
        number_lines,
        number_nonblank_lines,
    })
}
