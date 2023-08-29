use crate::Column::*;
use clap::Parser;
use std::{
    cmp::Ordering::*,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

/// Compare sorted files FILE1 and FILE2 line by line.
#[derive(Debug, Parser)]
#[command(
    name = "commr",
    about = "Rust comm",
    version = "0.1.0",
    author = "huzwares <huzwares@skiff.com>"
)]
pub struct Config {
    #[arg(value_name = "FILE1")]
    file1: String,

    #[arg(value_name = "FILE2")]
    file2: String,

    /// Suppress printing of column 1 (lines unique to FILE1).
    #[arg(short = '1', action = clap::ArgAction::SetFalse)]
    show_col1: bool,

    /// Suppress printing of column 2 (lines unique to FILE2).
    #[arg(short = '2', action = clap::ArgAction::SetFalse)]
    show_col2: bool,

    /// Suppress printing of column 3 (lines that appear in both files).
    #[arg(short = '3', action = clap::ArgAction::SetFalse)]
    shpw_col3: bool,

    /// Case-insensitive comparison of lines
    #[arg(short)]
    insensitive: bool,

    /// Output delimiter
    #[arg(
        short,
        long = "output-delimiter",
        value_name = "DELIM",
        default_value = "\t"
    )]
    delimiter: String,
}

#[derive(Debug)]
enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

pub fn get_args() -> MyResult<Config> {
    let conf = Config::parse();
    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    let file1 = &conf.file1;
    let file2 = &conf.file2;

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let case = |line: String| {
        if conf.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let print = |col: Column| {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if conf.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if conf.show_col2 {
                    if conf.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if conf.shpw_col3 {
                    if conf.show_col1 {
                        columns.push("");
                    }
                    if conf.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        };

        if !columns.is_empty() {
            println!("{}", columns.join(&conf.delimiter));
        }
    };

    let mut lines1 = open(file1)?.lines().map_while(Result::ok).map(case);
    let mut lines2 = open(file2)?.lines().map_while(Result::ok).map(case);

    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Equal => {
                    print(Col3(val1));
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
                Less => {
                    print(Col1(val1));
                    line1 = lines1.next();
                }
                Greater => {
                    print(Col2(val2));
                    line2 = lines2.next();
                }
            },
            (Some(val1), None) => {
                print(Col1(val1));
                line1 = lines1.next();
            }
            (None, Some(val2)) => {
                print(Col2(val2));
                line2 = lines2.next();
            }
            (None, None) => (),
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
