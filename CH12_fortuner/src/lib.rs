use clap::Parser;
use rand::prelude::*;
use regex::{Regex, RegexBuilder};
use std::{error::Error, ffi::OsStr, fs, path::PathBuf};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(
    name = "fortuner",
    version = "0.1.0",
    about = "Rust fortune",
    author = "huzwares <huzwares@skiff.com>"
)]
pub struct Config {
    /// Input files or directories
    #[arg(value_name = "FILE", required = true)]
    sources: Vec<String>,

    /// Pattern
    #[arg(short = 'm', long, value_name = "PATTERN")]
    pattern: Option<Regex>,

    /// Case-insensitive pattern matching
    #[arg(short, long, value_name = "CASE_INSENSITIVE")]
    insensitive: bool,

    /// Random seed
    #[arg(short, long, value_name = "SEED")]
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let mut conf = Config::parse();

    if conf.insensitive && conf.pattern.is_some() {
        conf.pattern = Some(
            RegexBuilder::new(conf.pattern.as_ref().unwrap().as_str())
                .case_insensitive(true)
                .build()
                .map_err(|_| format!("invalid value \'{}\'", conf.pattern.unwrap().as_str()))?,
        );
    }

    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    let files = find_files(&conf.sources)?;
    let fortunes = read_fortunes(&files)?;
    if let Some(pattern) = conf.pattern {
        let matched_fortune: Vec<_> = fortunes
            .iter()
            .filter(|&f| pattern.is_match(&f.text))
            .collect();
        let mut prev_source = None;
        for f in matched_fortune {
            if prev_source.as_ref().map_or(true, |s| s != &f.source) {
                eprintln!("({})\n%", f.source);
                prev_source = Some(f.source.clone());
            }
            println!("{}\n%", f.text);
        }
    } else {
        println!(
            "{}",
            pick_fortune(&fortunes, conf.seed)
                .or_else(|| Some("No fortunes found".to_string()))
                .unwrap()
        );
    }
    Ok(())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut res = vec![];
    for path in paths {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file()
                && !entry.file_name().eq_ignore_ascii_case(".DS_Store")
                && entry.path().extension() != Some(OsStr::new("dat"))
            {
                res.push(entry.into_path());
            }
        }
    }
    res.sort();
    res.dedup();
    Ok(res)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut res = vec![];
    for path in paths {
        let content = fs::read_to_string(path)?;
        content
            .split('%')
            .filter(|&s| !s.trim().is_empty())
            .for_each(|s| {
                res.push(Fortune {
                    source: path.file_name().unwrap().to_string_lossy().into_owned(),
                    text: s.trim().to_string(),
                })
            });
    }

    Ok(res)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    match seed {
        None => {
            let mut rng = thread_rng();
            let s = fortunes.choose(&mut rng)?;
            Some(s.text.to_string())
        }
        Some(sd) => {
            let mut rng = StdRng::seed_from_u64(sd);
            let s = fortunes.choose(&mut rng)?;
            Some(s.text.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{find_files, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());
        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());
        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));
        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());
        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }
        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                          attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];
        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
