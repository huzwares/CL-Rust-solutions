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
    version = "0.2.0",
    about = "Rust uniq",
    author = "huzwares <huzwares@skiff.com>"
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

    /// ignore differences in case when comparing
    #[arg(short = 'i', long, value_name = "IGNORE_CASE", default_value = "false", action = clap::ArgAction::SetTrue)]
    ignore_case: bool,

    /// compare no more than N character in lines
    #[arg(short = 'w', long, value_name = "CHECK_CHARS", default_value = None, action = clap::ArgAction::Set)]
    check_chars: Option<usize>,
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
        if check(&line, &previous, &conf.ignore_case, &conf.check_chars) {
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

fn check(current: &str, prev: &str, ignore_case: &bool, check_chars: &Option<usize>) -> bool {
    match (*ignore_case, *check_chars) {
        (true, Some(n)) => {
            current
                .trim_end()
                .to_lowercase()
                .chars()
                .take(n)
                .collect::<String>()
                != prev
                    .trim_end()
                    .to_lowercase()
                    .chars()
                    .take(n)
                    .collect::<String>()
        }
        (true, None) => current.trim_end().to_lowercase() != prev.trim_end().to_lowercase(),
        (false, Some(n)) => {
            current.trim_end().chars().take(n).collect::<String>()
                != prev.trim_end().chars().take(n).collect::<String>()
        }
        (false, None) => current.trim_end() != prev.trim_end(),
    }
}
