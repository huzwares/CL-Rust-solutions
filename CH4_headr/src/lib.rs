use clap::{Arg, Command};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .author("Me <Me@me.com>")
        .about("Rust head")
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
            Arg::new("lines")
                .long("lines")
                .short('n')
                .value_name("LINES")
                .help("print the first K lines instead of the first 10")
                .action(clap::ArgAction::Set)
                .required(false)
                // .value_parser(clap::value_parser!(usize))
                .default_value("10")
                .conflicts_with("bytes"),
        )
        .arg(
            Arg::new("bytes")
                .long("bytes")
                .short('c')
                .value_name("BYTES")
                .required(false)
                .action(clap::ArgAction::Set)
                .help("print the first K bytes of each file"),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("files")
        .unwrap()
        .map(|s| s.as_str().to_string())
        .collect();
    // let lines: usize = *matches.get_one::<usize>("lines").unwrap();
    // let lines = match parse_positive_int(matches.get_one::<String>("lines").unwrap()) {
    //     Ok(n) => n,
    //     Err(e) => {
    //         return Err(From::from(format!("illegal line count -- {}", e)));
    //     }
    // };
    let lines = matches
        .get_one::<String>("lines")
        .map(|s| s.as_str())
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();
    // let bytes = match matches.get_one::<String>("bytes") {
    //     Some(n) => match parse_positive_int(n) {
    //         Ok(n) => Some(n),
    //         Err(e) => {
    //             return Err(From::from(format!("illegal byte count -- {}", e)));
    //         }
    //     },
    //     None => None,
    // };
    let bytes = matches
        .get_one::<String>("bytes")
        .map(|s| s.as_str())
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;
    Ok(Config {
        files,
        lines,
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let len = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(mut file) => {
                if len > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                if let Some(n_bytes) = config.bytes {
                    let bytes = file.bytes().take(n_bytes).collect::<Result<Vec<_>, _>>();
                    print!("{}", String::from_utf8_lossy(&bytes?));
                } else {
                    let mut line = String::new();
                    for _ in 0..config.lines {
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

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
