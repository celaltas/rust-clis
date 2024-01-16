use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type CatResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> CatResult<Config> {
    let file_args = Arg::with_name("files")
        .value_name("FILE")
        .help("input file(s)")
        .multiple(true)
        .default_value("-");
    let can_show_number_lines_arg = Arg::with_name("can_show_number_lines")
        .short("n")
        .long("number")
        .help("print number lines")
        .takes_value(false)
        .conflicts_with("can_show_number_nonblank_lines");
    let can_show_number_nonblank_lines_arg = Arg::with_name("can_show_number_nonblank_lines")
        .short("b")
        .long("number-nonblank")
        .help("print number nonblank lines")
        .takes_value(false);

    let matches = App::new("catr")
        .version("0.0.1")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust cat")
        .arg(file_args)
        .arg(can_show_number_lines_arg)
        .arg(can_show_number_nonblank_lines_arg)
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("can_show_number_lines"),
        number_nonblank_lines: matches.is_present("can_show_number_nonblank_lines"),
    })
}

pub fn run(config: Config) -> CatResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed open to {}: {}", filename, err),
            Ok(file) => {
                if config.number_lines {
                    let mut line_number = 1;
                    for line_result in file.lines() {
                        let line = line_result?;
                        println!("{} {}", line_number, line);
                        line_number += 1;
                    }
                } else if config.number_nonblank_lines {
                    let mut line_number = 1;
                    for line_result in file.lines() {
                        let line = line_result?;
                        if !line.is_empty() {
                            println!("{}\t{}", line_number, line);
                            line_number += 1;
                        }else {
                            println!()
                        }
                    }
                } else {
                    for line_result in file.lines() {
                        let line = line_result?;
                        println!("{}", line);
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> CatResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
