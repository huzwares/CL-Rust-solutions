use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use itertools::izip;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Debug, Parser)]
#[command(
    name = "calr",
    about = "Rust cal",
    version = "0.2.0",
    author = "huzwares <huzwares@skiff.com>"
)]
pub struct Config {
    /// Month name or number (1-12)
    #[arg(short, value_name = "MONTH", value_parser = parse_month)]
    month: Option<u32>,

    /// Year (1-9999)
    #[arg(value_name = "YEAR")]
    year: Option<i32>,

    /// Show whole current year
    #[arg(short = 'y', long = "year", value_name = "SHOW_YEAR", conflicts_with_all = ["year", "month"])]
    show_current_year: bool,

    #[arg(hide = true, required = false)]
    today: Option<NaiveDate>,

    /// Show alternative display
    #[arg(long, value_name = "NCAL", conflicts_with_all = ["show_current_year", "year"])]
    ncal: bool,
}

pub fn get_args() -> MyResult<Config> {
    let mut conf = Config::parse();
    let today = Local::now();
    if conf.show_current_year {
        conf.month = None;
        conf.year = Some(today.year());
    } else if conf.month.is_none() && conf.year.is_none() {
        conf.month = Some(today.month());
        conf.year = Some(today.year());
    } else if conf.month.is_some() && conf.year.is_none() {
        conf.year = Some(today.year());
    }

    if !(1..=9999).contains(&conf.year.unwrap()) {
        return Err(From::from(format!(
            "year \"{}\" not in the range 1 through 9999",
            conf.year.unwrap()
        )));
    }
    conf.today = Some(today.date_naive());
    Ok(conf)
}

pub fn run(conf: Config) -> MyResult<()> {
    match conf.month {
        Some(month) => {
            let lines = format_month(conf.year.unwrap(), month, true, conf.today.unwrap());
            if conf.ncal {
                let mut offset_for_reversed_day = 0;
                let check_curtent_day_exist =
                    |l: &String| l.contains(&Style::new().reverse().prefix().to_string());
                println!("{}", lines[0]);
                for i in 0..9 {
                    let n_line = lines[1..]
                        .iter()
                        .map(|line| {
                            if check_curtent_day_exist(line) {
                                if line
                                    .chars()
                                    .skip(i * 3)
                                    .take(3)
                                    .collect::<String>()
                                    .contains("[7")
                                {
                                    offset_for_reversed_day = 8;
                                    line.chars().skip(i * 3).take(11).collect::<String>()
                                } else {
                                    line.chars()
                                        .skip(i * 3 + offset_for_reversed_day)
                                        .take(3)
                                        .collect::<String>()
                                }
                            } else {
                                line.chars().skip(i * 3).take(3).collect::<String>()
                            }
                        })
                        .collect::<String>();
                    println!("{}", n_line);
                }
            } else {
                println!("{}", lines.join("\n"));
            }
        }
        None => {
            println!("{:>32}", conf.year.unwrap());
            let months: Vec<_> = (1..=12)
                .map(|month| format_month(conf.year.unwrap(), month, false, conf.today.unwrap()))
                .collect();

            for (i, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for lines in izip!(m1, m2, m3) {
                        println!("{}{}{}", lines.0, lines.1, lines.2);
                    }
                    if i < 3 {
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_month(month: &str) -> MyResult<u32> {
    if let Ok(n) = month.parse::<u32>() {
        if (1..=12).contains(&n) {
            return Ok(n);
        } else {
            return Err(From::from(format!(
                "month \"{}\" not in the range 1 through 12",
                n
            )));
        }
    }
    let tmp = MONTH_NAMES
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            if name.to_lowercase().starts_with(&month.to_lowercase()) {
                Some(i + 1)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if tmp.len() == 1 {
        return Ok(tmp[0] as u32);
    }
    Err(From::from(format!("Invalid month \"{}\"", month)))
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days = (1..first.weekday().number_from_sunday())
        .map(|_| "  ".to_string())
        .collect::<Vec<String>>();

    let is_today = |day: u32| year == today.year() && month == today.month() && day == today.day();

    let last = last_day_in_month(year, month);
    days.extend((first.day()..=last.day()).map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));

    let mut res = Vec::with_capacity(8);
    let month_name = MONTH_NAMES[(month - 1) as usize];
    res.push(format!(
        "{:^20}  ",
        if print_year {
            format!("{} {}", month_name, year)
        } else {
            month_name.to_string()
        },
    ));
    res.push("Su Mo Tu We Th Fr Sa  ".to_string());

    for week in days.chunks(7) {
        res.push(format!("{:20}  ", week.join(" ")));
    }

    while res.len() < 8 {
        res.push(" ".repeat(22));
    }

    res
    // let mut days: Vec<String> = (1..first
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::{format_month, parse_month, NaiveDate};

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);
        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );
        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );
        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }
    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }
}
