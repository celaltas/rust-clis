use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type WCResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> WCResult<Config> {
    let file_args = Arg::with_name("files")
        .value_name("FILE")
        .help("input file(s)")
        .multiple(true)
        .default_value("-");
    let lines = Arg::with_name("lines")
        .short("l")
        .long("lines")
        .help("Show line count")
        .takes_value(false);
    let words = Arg::with_name("words")
        .short("w")
        .long("words")
        .help("Show word count")
        .takes_value(false);
    let bytes = Arg::with_name("bytes")
        .short("c")
        .long("bytes")
        .help("Show byte count")
        .takes_value(false);
    let chars = Arg::with_name("chars")
        .short("m")
        .long("chars")
        .help("Show character count")
        .conflicts_with("bytes")
        .takes_value(false);

    let matches = App::new("wcr")
        .version("0.0.1")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust wc")
        .arg(file_args)
        .arg(lines)
        .arg(words)
        .arg(bytes)
        .arg(chars)
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if [lines, words, bytes, chars].iter().all(|v| !v) {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn run(config: Config) -> WCResult<()> {
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &config.files {
        match open(filename) {
            Ok(file) => {
                let info = count(file)?;
                println!(
                    "{}{}{}{}{}",
                    format_field(info.num_lines, config.lines),
                    format_field(info.num_words, config.words),
                    format_field(info.num_bytes, config.bytes),
                    format_field(info.num_chars, config.chars),
                    if filename == "-" {
                        "".to_string()
                    } else {
                        format!(" {}", filename)
                    }
                );
                total_lines += info.num_lines;
                total_words += info.num_words;
                total_bytes += info.num_bytes;
                total_chars += info.num_chars;
            }
            Err(err) => eprintln!("head: {}: {}", filename, err),
        }
    }
    if config.files.len() > 1 {
        println!(
            "{}{}{}{} total",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_bytes, config.bytes),
            format_field(total_chars, config.chars)
        );
    }
    Ok(())
}

fn open(filename: &str) -> WCResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> WCResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut buf = String::new();

    loop {
        let byte_read = file.read_line(&mut buf)?;
        if byte_read == 0 {
            break;
        } else {
            num_lines += 1;
            num_bytes += byte_read;
            num_chars += buf.chars().count();
            num_words += buf.split_whitespace().count();
            buf.clear()
        }
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {

    use super::{count, format_field, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert!(info.is_ok());
        assert_eq!(info.unwrap(), expected);
    }

    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "        3");
        assert_eq!(format_field(10, true), "       10");
    }
}
