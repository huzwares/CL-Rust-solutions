use crate::EntryType::*;
use clap::{Parser, ValueEnum};
use regex::Regex;
use std::error::Error;
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(
    version = "0.2.0",
    author = "huzwares <huzwares@skiff.com>",
    name = "findr",
    about = "Rust find"
)]
pub struct Config {
    /// Serach paths
    #[arg(value_name = "PATH", default_value = ".")]
    path: Vec<String>,

    /// Name
    #[arg(short, long, value_name = "NAME", num_args = 1..)]
    name: Vec<Regex>,

    /// Entry type
    #[arg(value_enum, short = 't', long = "type", value_name = "TYPE", num_args = 1..)]
    entry_type: Vec<EntryType>,

    /// Show counter
    #[arg(short = 'c', long, value_name = "COUNT")]
    count: bool,

    /// Set max depth
    #[arg(long, value_name = "MAX_DEPTH")]
    max_depth: Option<usize>,

    /// Set min depth
    #[arg(long, value_name = "MIN_DEPTH")]
    min_depth: Option<usize>,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum EntryType {
    #[value(alias = "d")]
    Dir,
    #[value(alias = "f")]
    File,
    #[value(alias = "l")]
    Link,
}

pub fn get_args() -> MyResult<Config> {
    let conf = Config::parse();
    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    let (mut link_count, mut dir_count, mut file_count) = (0, 0, 0);
    let type_filter = |e: &DirEntry| {
        conf.entry_type.is_empty()
            || conf.entry_type.iter().any(|t| match t {
                Link => e.file_type().is_symlink(),
                Dir => e.file_type().is_dir(),
                File => e.file_type().is_file(),
            })
    };
    let name_filter = |e: &DirEntry| {
        conf.name.is_empty()
            || conf
                .name
                .iter()
                .any(|r| r.is_match(&e.file_name().to_string_lossy()))
    };
    for path in &conf.path {
        let entries = WalkDir::new(path)
            .min_depth(conf.min_depth.unwrap_or(0))
            .max_depth(conf.max_depth.unwrap_or(usize::MAX))
            .into_iter()
            .filter_map(|res| match res {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
                Ok(entry) => Some(entry),
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| {
                match entry.file_type() {
                    e if e.is_dir() => {
                        dir_count += 1;
                    }
                    e if e.is_file() => {
                        file_count += 1;
                    }
                    e if e.is_symlink() => {
                        link_count += 1;
                    }
                    _ => (),
                }
                entry.path().display().to_string()
            })
            .collect::<Vec<_>>();
        println!("{}", entries.join("\n"));
        if conf.count {
            println!("--------- counter ---------");
            if dir_count > 0 {
                println!("{dir_count} directories");
            }
            if file_count > 0 {
                println!("{file_count} files");
            }
            if link_count > 0 {
                println!("{link_count} links");
            }
        }
        // for entry in WalkDir::new(path) {
        //     match entry {
        //         Err(e) => eprintln!("{}", e),
        //         Ok(entry) => {
        //             let file_type = if entry.file_type().is_dir() {
        //                 Dir
        //             } else if entry.file_type().is_file() {
        //                 File
        //             } else {
        //                 Link
        //             };
        //             if (conf.entry_type.is_empty() || conf.entry_type.contains(&file_type))
        //                 && (conf.name.is_empty()
        //                     || conf
        //                         .name
        //                         .iter()
        //                         .any(|r| r.is_match(&entry.file_name().to_string_lossy())))
        //             {
        //                 println!("{}", entry.path().display())
        //             }
        //         }
        //     }
        // }
    }
    Ok(())
}
