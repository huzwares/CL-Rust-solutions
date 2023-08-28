use clap::Parser;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Range,
};

type MyResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

type PositionList = Vec<Range<usize>>;

#[derive(Debug, Parser)]
#[command(
    about = "Rust cut",
    version = "0.1.0",
    author = "huzwares <huzwares@skiff.com>",
    name = "cutr"
)]
pub struct Config {
    /// Input file(s)
    #[arg(value_name = "FILES", default_value = "-")]
    files: Vec<String>,

    /// Field delimiter
    #[arg(short, long, value_name = "DELIMITER", default_value = "\t")]
    delimiter: char,

    /// Selected fields
    #[arg(short, long, value_name = "FILEDS", conflicts_with_all = ["bytes", "chars"], value_parser = parse_pos)]
    fields: Option<PositionList>,

    /// Selected bytes
    #[arg(short, long, value_name = "BYTES", conflicts_with = "chars", value_parser = parse_pos)]
    bytes: Option<PositionList>,

    /// Selected characters
    #[arg(short, long, value_name = "CHARS", value_parser = parse_pos)]
    chars: Option<PositionList>,
}

pub fn get_args() -> MyResult<Config> {
    let conf = Config::parse();
    if conf.fields.is_none() && conf.bytes.is_none() && conf.chars.is_none() {
        return Err(From::from("Must have --fields, --bytes, or --chars"));
    }
    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    for filename in conf.files {
        match open(&filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                if let Some(p) = &conf.chars {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, p));
                    }
                } else if let Some(p) = &conf.bytes {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, p));
                    }
                } else if let Some(p) = &conf.fields {
                    let mut reader = ReaderBuilder::new()
                        .delimiter(conf.delimiter as u8)
                        .has_headers(false)
                        .from_reader(file);
                    let mut wtr = WriterBuilder::new()
                        .delimiter(conf.delimiter as u8)
                        .from_writer(io::stdout());
                    for record in reader.records() {
                        let record = record?;
                        wtr.write_record(extract_fields(&record, p))?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn parse_pos(range: &str) -> MyResult<PositionList> {
    let mut result: Vec<Range<usize>> = vec![];
    let ranges: Vec<&str> = range.split(',').collect();
    for range in ranges {
        if range.contains('+') || range.contains(char::is_alphabetic) {
            return Err(From::from(format!("illegal list value: \"{range}\"")));
        }
        let temp: Vec<&str> = range.split('-').collect();
        let mut start: usize;
        let end: usize;
        match temp.len() {
            1 => {
                end = match temp[0].parse::<usize>() {
                    Ok(n) if n > 0 => n,
                    _ => {
                        return Err(From::from(format!("illegal list value: \"{}\"", temp[0])));
                    }
                };
                start = end - 1;
            }
            2 => {
                start = match temp[0].parse::<usize>() {
                    Ok(n) if n > 0 => n,
                    _ => {
                        return Err(From::from(format!("illegal list value: \"{}\"", temp[0])));
                    }
                };
                end = match temp[1].parse::<usize>() {
                    Ok(n) if n > 0 => n,
                    _ => {
                        return Err(From::from(format!("illegal list value: \"{}\"", temp[1])));
                    }
                };
                if end <= start {
                    return Err(From::from(format!(
                        "First number in range ({start}) must be lower than second number ({end})"
                    )));
                }

                start -= 1;
            }
            _ => {
                return Err(From::from("illegal list values"));
            }
        }
        result.push(Range { start, end });
    }
    Ok(result)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    char_pos
        .iter()
        .map(|r| {
            line.chars()
                .skip(r.start)
                .take(r.end - r.start)
                .collect::<String>()
        })
        .collect()
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    byte_pos
        .iter()
        .map(|r| {
            String::from_utf8_lossy(
                line.bytes()
                    .skip(r.start)
                    .take(r.end - r.start)
                    .collect::<Vec<u8>>()
                    .as_slice(),
            )
            .into_owned()
        })
        .collect()
}

fn extract_fields<'a>(record: &'a StringRecord, filed_pos: &[Range<usize>]) -> Vec<&'a str> {
    filed_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| record.get(i)))
        .collect()
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, parse_pos};

    #[test]
    fn test_parse_pos() {
        assert!(parse_pos("").is_err());

        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"");

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"");

        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"");

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"");

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"");

        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"");

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"");

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"");

        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
}
