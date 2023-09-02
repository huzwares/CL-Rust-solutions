use chrono::prelude::*;
use clap::Parser;
use std::{error::Error, fs, os::unix::fs::MetadataExt, path::PathBuf};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(
    name = "lsr",
    version = "0.1.0",
    about = "Rust ls",
    author = "huzwares <huzwares@skiff.com>"
)]
pub struct Config {
    /// Files and/or directories
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    /// Long listing
    #[arg(short, long, value_name = "LONG")]
    long: bool,

    /// Show all files
    #[arg(short = 'a', long = "all", value_name = "SHOW_ALL")]
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    let conf = Config::parse();
    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    let paths = find_files(&conf.paths, conf.show_hidden)?;
    if conf.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut res = vec![];
    for path in paths {
        match fs::metadata(path) {
            Ok(meta) => {
                if meta.is_file() {
                    res.push(PathBuf::from(path));
                    continue;
                }
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let f_name = entry.file_name().to_string_lossy().into_owned();
                    let f_path = entry.path();
                    if f_name.starts_with('.') && !show_hidden {
                        continue;
                    }
                    res.push(f_path);
                }
            }
            Err(e) => {
                eprintln!("{}: {}", path, e);
                continue;
            }
        }
    }
    Ok(res)
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);
    for path in paths {
        let meta = fs::metadata(path)?;
        table.add_row(
            Row::new()
                .with_cell(if meta.is_dir() { "d" } else { "-" })
                .with_cell(format_mode(meta.mode()))
                .with_cell(meta.nlink())
                .with_cell(format!(
                    "{:?}",
                    get_user_by_uid(meta.uid())
                        .map(|u| u.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| meta.uid().to_string())
                ))
                .with_cell(format!(
                    "{:?}",
                    get_group_by_gid(meta.gid())
                        .map(|g| g.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| meta.gid().to_string())
                ))
                .with_cell(meta.len())
                .with_cell(format!(
                    "{}",
                    DateTime::<Local>::from(meta.modified()?).format("%b %d %y %H:%M")
                ))
                .with_cell(path.display()),
        );
    }

    Ok(format!("{}", table))
}

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
fn format_mode(mode: u32) -> String {
    [
        (0o400_u32, "r"),
        (0o200_u32, "w"),
        (0o100_u32, "x"),
        (0o040_u32, "r"),
        (0o020_u32, "w"),
        (0o010_u32, "x"),
        (0o004_u32, "r"),
        (0o002_u32, "x"),
        (0o001_u32, "x"),
    ]
    .into_iter()
    .map(|(code, out)| match_permission(mode, code, out))
    .collect::<String>()
}

fn match_permission(mode: u32, code: u32, out: &str) -> String {
    match mode & code == code {
        true => out.to_string(),
        false => "-".to_string(),
    }
}

#[allow(dead_code)]
fn long_match(line: &str, expected_name: &str, expected_perms: &str, expected_size: Option<&str>) {
    let parts: Vec<_> = line.split_whitespace().collect();
    assert!(!parts.is_empty() && parts.len() <= 10);
    let perms = parts.first().unwrap();
    assert_eq!(perms, &expected_perms);
    if let Some(size) = expected_size {
        let file_size = parts.get(4).unwrap();
        assert_eq!(file_size, &size);
    }
    let display_name = parts.last().unwrap();
    assert_eq!(display_name, &expected_name);
}

#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output, long_match, PathBuf};
    #[test]
    fn test_find_files() {
        // Find all nonhidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Find all entries in a directory
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);
        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }
    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);
        let res = format_output(&[bustle]);
        assert!(res.is_ok());
        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);
        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());
        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);
        let empty_line = lines.remove(0);
        long_match(
            &empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );
        let dir_line = lines.remove(0);
        long_match(&dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }
}
