use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

/// report or filter out repeated lines in a file
#[derive(Parser, Debug)]
#[command(
    name = "uniqr",
    version = "0.1.0",
    about = "Rust uniq",
    author = "ME <me@me.com"
)]
pub struct Config {
    /// Input file
    #[arg(value_name = "IN_FILE", default_value = "-")]
    in_file: String,

    /// Output file
    #[arg(value_name = "OUT_FILE", default_value = None)]
    out_file: Option<String>,

    /// Show counts
    #[arg(short = 'c', long, value_name = "COUNT", action = clap::ArgAction::SetTrue)]
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let conf = Config::parse();

    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    let mut file = open(&conf.in_file).map_err(|e| format!("{}: {}", conf.in_file, e))?;
    let mut out_file: Box<dyn Write> = match &conf.out_file {
        Some(path) => Box::new(File::create(path)?),
        _ => Box::new(io::stdout()),
    };
    let mut print = |count: u64, text: &str| -> MyResult<()> {
        if count > 0 {
            if conf.count {
                write!(out_file, "{:>4} {}", count, text)?;
            } else {
                write!(out_file, "{}", text)?;
            }
        }
        Ok(())
    };
    let mut line = String::new();
    let mut previous = String::new();
    let mut count: u64 = 0;
    while file.read_line(&mut line)? != 0 {
        if line.trim_end() != previous.trim_end() {
            print(count, &previous)?;
            previous = line.clone();
            count = 0;
        }
        count += 1;
        line.clear();
    }
    print(count, &previous)?;
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
