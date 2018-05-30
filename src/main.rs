extern crate ansi_term;
#[macro_use]
extern crate clap;

use ansi_term::Colour::{Green, Red};
use clap::{App, Arg};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error};
use std::process::exit;

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Does awesome things")
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

    let lines = matches
        .value_of("lines")
        .unwrap_or("10")
        .parse::<i32>()
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            exit(1);
        });

    let quiet = matches.is_present("quiet");

    for file in files {
        head(file, lines, quiet)?;
    }
    Ok(())
}

fn head(file: Option<&str>, lines: i32, quiet: bool) -> Result<(), Error> {
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
    for line in reader.lines() {
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
    Ok(())
}
