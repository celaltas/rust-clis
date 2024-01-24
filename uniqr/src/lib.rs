use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
};

use clap::{App, Arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    infile: String,
    outfile: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let infile_arg = Arg::with_name("infile")
        .value_name("INPUT FILE")
        .help("input file")
        .multiple(false)
        .required(true)
        .default_value("-");
    let output_arg = Arg::with_name("outfile")
        .value_name("OUTPUT FILE")
        .help("output file")
        .multiple(false);
    let count = Arg::with_name("count")
        .short("c")
        .long("count")
        .help("show counts")
        .takes_value(false);
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust uniq")
        .arg(infile_arg)
        .arg(output_arg)
        .arg(count)
        .get_matches();

    Ok(Config {
        infile: matches.value_of("infile").unwrap().to_string(),
        outfile: matches.value_of("outfile").map(String::from),
        count: matches.is_present("count"),
    })
}

pub fn run(conf: Config) -> MyResult<()> {
    let mut file = open(&conf.infile).map_err(|e| format!("{}:{}", conf.infile, e))?;
    let mut line = String::new();
    let mut previous = String::new();
    let mut count: u64 = 0;
    let mut outfile: Box<dyn Write> = match &conf.outfile {
        Some(path) => Box::new(File::create(path)?),
        _ => Box::new(io::stdout()),
    };

    let mut print = |count: u64, text: &str| -> MyResult<()> {
        if count > 0 {
            if conf.count {
                write!(outfile, "{:>4} {}", count, text)?;
            } else {
                write!(outfile, "{}", text)?;
            }
        };
        Ok(())
    };

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if line.trim_end() != previous.trim_end() {
            print(count, &previous)?;
            previous = line.clone();
            count = 0;
        }
        count += 1;
        line.clear();
    }
    print(count, &previous)?;

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
