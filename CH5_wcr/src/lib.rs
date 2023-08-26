use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(
    name = "wcr",
    version = "0.1.0",
    about = "Rust wc",
    author = "huzwares <huzwares@skiff.com>"
)]
pub struct Config {
    /// Input files
    #[arg(value_name = "FILES", default_value = "-")]
    files: Vec<String>,

    /// Show line count
    #[arg(short = 'l', long, value_name = "LINES", action = clap::ArgAction::SetTrue)]
    lines: bool,

    /// Show word count
    #[arg(short = 'w', long, value_name = "WORDS", action = clap::ArgAction::SetTrue)]
    words: bool,

    /// Show bytes count
    #[arg(short = 'c', long, value_name = "BYTES", conflicts_with = "chars", action = clap::ArgAction::SetTrue)]
    bytes: bool,

    /// Show character count
    #[arg(short = 'm', long, value_name = "CHARS", action = clap::ArgAction::SetTrue)]
    chars: bool,

    /// Print the length of the longest line
    #[arg(short = 'L', long = "max-line-length", value_name = "MAX-LINE-LENGHT", action = clap::ArgAction::SetTrue)]
    max_line_length: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
    max_line_length: usize,
}

pub fn get_args() -> MyResult<Config> {
    let mut config = Config::parse();

    if !config.lines && !config.words && !config.bytes && !config.chars && !config.max_line_length {
        config.lines = true;
        config.words = true;
        config.bytes = true;
    }

    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total = FileInfo {
        num_lines: 0,
        num_words: 0,
        num_bytes: 0,
        num_chars: 0,
        max_line_length: 0,
    };
    for filename in &config.files {
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                let info = count(file)?;
                total.num_lines += info.num_lines;
                total.num_words += info.num_words;
                total.num_bytes += info.num_bytes;
                total.num_chars += info.num_chars;
                if config.lines {
                    print!("{:>8}", info.num_lines);
                }
                if config.words {
                    print!("{:>8}", info.num_words);
                }
                if config.chars {
                    print!("{:>8}", info.num_chars);
                }
                if config.bytes {
                    print!("{:>8}", info.num_bytes);
                }
                if config.max_line_length {
                    print!("{:>8}", info.max_line_length);
                }
                println!(" {}", if filename == "-" { "" } else { filename });
            }
        }
    }
    if config.files.len() > 1 {
        if config.lines {
            print!("{:>8}", total.num_lines);
        }
        if config.words {
            print!("{:>8}", total.num_words);
        }
        if config.chars {
            print!("{:>8}", total.num_chars);
        }
        if config.bytes {
            print!("{:>8}", total.num_bytes);
        }
        if config.max_line_length {
            println!("{:>8}", total.max_line_length);
        }
        println!(" total");
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    let num_lines = input.lines().count();
    let num_words = input.split_whitespace().count();
    let num_bytes = input.bytes().count();
    let num_chars = input.chars().count();
    let mut max_line_length = 0;
    for line in input.lines() {
        if line.chars().count() > max_line_length {
            max_line_length = line.chars().count();
        }
    }
    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
        max_line_length,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
            max_line_length: 48,
        };

        assert_eq!(info.unwrap(), expected);
    }
}
