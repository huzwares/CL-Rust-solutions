use clap::Parser;
use colored::*;
use regex::{Regex, RegexBuilder};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    mem,
    ops::Range,
};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(
    name = "grepr",
    about = "Rust grep",
    author = "huzwares <huzwares@skiff.com>",
    version = "0.2.0"
)]
pub struct Config {
    /// Search pattern
    #[arg(value_name = "PATTERN")]
    pattern: Regex,

    /// Input file(s)
    #[arg(value_name = "FILES", default_value = "-")]
    files: Vec<String>,

    /// Recursive search
    #[arg(short, long, value_name = "RECURSIVE")]
    recursive: bool,

    /// Count occurrences
    #[arg(short, long, value_name = "COUNT")]
    count: bool,

    /// Invert match
    #[arg(short = 'v', long, value_name = "INVER_MATCH")]
    invert_match: bool,

    /// Case-insensitive
    #[arg(short = 'i', long = "insensitive", value_name = "CASE_INSENSITIVE")]
    case_insensitive: bool,
}

pub fn get_args() -> MyResult<Config> {
    let mut conf = Config::parse();
    conf.pattern = RegexBuilder::new(conf.pattern.as_str())
        .case_insensitive(conf.case_insensitive)
        .build()
        .map_err(|_| format!("invalid value \'{}\'", conf.pattern.as_str()))?;
    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    let entries = find_files(&conf.files, conf.recursive);
    let len = entries.len();
    let print = |filename: &str, val: &str, r: Range<usize>| {
        if r.is_empty() {
            if len > 1 {
                print!("{}:{}", filename.bold(), val);
            } else {
                print!("{}", val);
            }
        } else {
            let (first, second) = val.split_at(r.start);
            let (second, third) = second.split_at(r.end - r.start);
            if len > 1 {
                print!(
                    "{}:{}{}{}",
                    filename.bold(),
                    first,
                    second.green().bold(),
                    third
                );
            } else {
                print!("{}{}{}", first, second.green().bold(), third);
            }
        }
    };
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e.to_string().red()),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename.bold(), e.to_string().red()),
                Ok(file) => match find_lines(file, &conf.pattern, conf.invert_match) {
                    Err(e) => eprintln!("{}", e.to_string().red()),
                    Ok(matches) => {
                        if conf.count {
                            print(&filename, &format!("{}\n", matches.len()), 0..0);
                        } else {
                            for (line, findes) in matches {
                                print(&filename, &line, findes);
                            }
                        }
                    }
                },
            },
        }
    }
    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut res = vec![];
    for path in paths {
        match path.as_str() {
            "-" => res.push(Ok(path.to_string())),
            _ => match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if recursive {
                            for entry in WalkDir::new(path)
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file())
                            {
                                res.push(Ok(entry.path().display().to_string()));
                            }
                        } else {
                            res.push(Err(From::from(format!("{} is a directory", path))));
                        }
                    } else if metadata.is_file() {
                        res.push(Ok(path.to_string()))
                    }
                }
                Err(e) => res.push(Err(From::from(format!("{}: {}", path.bold(), e)))),
            },
        }
    }
    res
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<(String, Range<usize>)>> {
    let mut result = vec![];
    let mut line = String::new();
    while file.read_line(&mut line)? != 0 {
        match invert_match {
            false => {
                // let r = pattern.find(&line).unwrap().range();
                if pattern.is_match(&line) {
                    let matches = pattern.find(&line).unwrap().range();
                    result.push((mem::take(&mut line), matches));
                }
            }
            true => {
                if !pattern.is_match(&line) {
                    let matches = 0..line.bytes().count();
                    result.push((mem::take(&mut line), matches));
                }
            }
        }
        // if pattern.is_match(&line) ^ invert_match {
        //     result.push((mem::take(&mut line), "".to_string()));
        // }
        line.clear();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;
    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");
        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }
        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );
        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";
        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
