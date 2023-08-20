use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(name = "headr")]
#[command(version = "0.1.0")]
#[command(about = "Rust head (extended)")]
pub struct Config {
    #[arg(value_name = "FILES", default_value = "-")]
    files: Vec<String>,

    #[arg(
        short = 'n',
        long,
        value_name = "LINES",
        default_value = "10",
        conflicts_with = "bytes"
    )]
    lines: usize,

    #[arg(short = 'c', long, value_name = "BYTES", action = clap::ArgAction::Set)]
    bytes: Option<usize>,

    #[arg(short = 'q', long = "quite", long = "silent", value_name = "SILENT", default_value = "false", action = clap::ArgAction::SetTrue, conflicts_with = "verbose")]
    silent: bool,

    #[arg(short = 'v', long = "verbose", value_name = "VERBOSE", default_value = "false", action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

pub fn get_args() -> MyResult<Config> {
    let cli = Config::parse();
    if cli.lines < 1 {
        return Err(From::from(format!(
            "invalid value '{}' for '--lines <LINES>': invalid digit found in string",
            cli.lines
        )));
    }
    if let Some(n) = cli.bytes {
        if n < 1 {
            return Err(From::from(format!(
                "invalid value '{}' for '--bytes <BYTES>': invalid digit found in string",
                n
            )));
        }
    }
    Ok(cli)
}

pub fn run(conf: Config) -> MyResult<()> {
    let len = conf.files.len();
    for (file_num, filename) in conf.files.iter().enumerate() {
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(mut file) => {
                if (len > 1 || conf.verbose) && !conf.silent {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                if let Some(n_bytes) = conf.bytes {
                    let bytes = file.bytes().take(n_bytes).collect::<Result<Vec<_>, _>>();
                    print!("{}", String::from_utf8_lossy(&mut bytes?));
                } else {
                    let mut line = String::new();
                    for _ in 0..conf.lines {
                        let bytes = file.read_line(&mut line)?;
                        if bytes == 0 {
                            break;
                        }
                        print!("{}", line);
                        line.clear();
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
