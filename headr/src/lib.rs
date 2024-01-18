use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

use clap::{App, Arg};

type HeadResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> HeadResult<Config> {
    let file_args = Arg::with_name("files")
        .value_name("FILE")
        .help("input file(s)")
        .multiple(true)
        .default_value("-");
    let line_arg = Arg::with_name("lines")
        .help("Number of lines")
        .short("n")
        .long("lines")
        .takes_value(true)
        .default_value("10");
    let byte_arg = Arg::with_name("bytes")
        .help("Number of bytes")
        .short("c")
        .long("bytes")
        .conflicts_with("lines")
        .takes_value(true);

    let matches = App::new("headr")
        .version("0.1.0")
        .author("celaltas <celal.tas123@gmail.com>")
        .about("Rust head")
        .arg(file_args)
        .arg(line_arg)
        .arg(byte_arg)
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let byte = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: files,
        lines: lines.unwrap(),
        bytes: byte,
    })
}

pub fn run(config: Config) -> HeadResult<()> {
    for filename in config.files {
        match open(&filename) {
            Ok(file) => {

                println!("==> {} <==", filename);

                if let Some(bytes_number) = config.bytes {
                    read_bytes(bytes_number, file)?;
                } else {
                    read_line(config.lines, file)?;
                }
            }
            Err(err) => eprintln!("head: {}: {}", filename, err),
        }
    }
    Ok(())
}

fn read_bytes(bytes_number: usize, file: Box<dyn BufRead>) -> Result<(), Box<dyn Error>> {
    let mut handle = file.take(bytes_number as u64);
    let mut buffer = vec![0; bytes_number];
    let bytes_read = handle.read(&mut buffer)?;
    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
    Ok(())
}

fn read_line(line_number: usize, mut file: Box<dyn BufRead>) -> Result<(), Box<dyn Error>> {
    let mut line = String::new();
    Ok(for _ in 0..line_number {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        print!("{}", line);
        line.clear()
    })
}

fn parse_positive_int(val: &str) -> HeadResult<usize> {
    match val.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from("invalid positive integer")),
    }
}

fn open(filename: &str) -> HeadResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());

    let res = parse_positive_int("0");
    assert!(res.is_err());
}
