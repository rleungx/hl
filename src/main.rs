extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate regex;

use ansi_term::Colour::{Green, Red};
use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error, Read};
use std::process::exit;

const UNIT: u64 = 1;
const DATA_MAGNITUDE: u64 = 1024;
const KB: u64 = UNIT * DATA_MAGNITUDE;
const MB: u64 = KB * DATA_MAGNITUDE;
const GB: u64 = MB * DATA_MAGNITUDE;
const TB: u64 = (GB as u64) * (DATA_MAGNITUDE as u64);
const PB: u64 = (TB as u64) * (DATA_MAGNITUDE as u64);

#[derive(Debug, Copy, Clone)]
enum Mode {
    Bytes(u64),
    Lines(i32),
}

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Yet another head")
        .arg(
            Arg::with_name("FILE")
                .help("file(s) to print.")
                .multiple(true)
                .takes_value(true)
                .empty_values(false),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help("print the first n lines")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("print the first n bytes")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("print without filename and line number"),
        )
        .get_matches();

    let files: Vec<Option<&str>> = matches
        .values_of("FILE")
        .map(|values| {
            values
                .map(|filename| {
                    if filename == "-" {
                        None
                    } else {
                        Some(filename)
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| vec![None]);

    let mode = if matches.is_present("bytes") {
        let bytes = matches.value_of("bytes").unwrap();
        let re = Regex::new(r"([1-9]\d*)([A-Za-z]+)").unwrap();
        let caps = re.captures(bytes).unwrap_or_else(|| {
            eprintln!("require a number of bytes, e.g., 10B");
            exit(1);
        });
        let byte = caps.get(1)
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                exit(1);
            }) as u64;

        let unit = match caps.get(2).unwrap().as_str().to_uppercase().as_str() {
            "K" | "KB" => KB,
            "M" | "MB" => MB,
            "G" | "GB" => GB,
            "T" | "TB" => TB,
            "P" | "PB" => PB,
            "B" => UNIT,
            _ => {
                eprintln!("only B, KB, MB, GB, TB, PB are supported");
                exit(1);
            }
        };
        Mode::Bytes(unit * byte)
    } else {
        let lines = matches
            .value_of("lines")
            .unwrap_or("10")
            .parse::<i32>()
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                exit(1);
            });
        Mode::Lines(lines)
    };

    let quiet = matches.is_present("quiet");

    for file in files {
        head(file, mode, quiet)?;
        println!();
    }
    Ok(())
}

fn head(file: Option<&str>, mode: Mode, quiet: bool) -> Result<(), Error> {
    let stdin = io::stdin();
    let reader: Box<BufRead> = match file {
        None => Box::new(stdin.lock()),
        Some(filename) => {
            if !quiet {
                println!("{}", Red.bold().paint(filename));
            }
            Box::new(BufReader::new(File::open(filename)?))
        }
    };

    let mut line_num = 1;

    match mode {
        Mode::Bytes(bytes) => {
            if !quiet {
                print!("{}: ", Green.paint(format!("{}", line_num)));
            }
            for byte in reader.bytes().take(bytes as usize) {
                let c = byte.unwrap() as char;
                print!("{}", c);
                if !quiet && c == '\n' {
                    line_num += 1;
                    print!("{}: ", Green.paint(format!("{}", line_num)));
                }
            }
        }
        Mode::Lines(lines) => {
            for line in reader.lines().take(lines as usize) {
                if line_num <= lines {
                    let l = line.unwrap();
                    if quiet {
                        println!("{}", l);
                    } else {
                        println!("{}: {}", Green.paint(format!("{}", line_num)), l);
                    }
                    line_num += 1;
                }
            }
        }
    }

    Ok(())
}
