use clap::{Arg, Command};

fn main() {
    let matches = Command::new("echor")
        .version("0.1.0")
        .author("ME")
        .about("Rust echo")
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .value_parser(clap::value_parser!(String))
                .help("Input text")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .help("Do not print newline")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let text = matches
        .get_many::<String>("text")
        .unwrap()
        .map(|s| s.as_str().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    let omit_value = matches.get_one::<bool>("omit_newline");
    match omit_value {
        Some(true) => {
            print!("{text}");
        }
        _ => {
            println!("{text}");
        }
    }
}
