use std::error::Error;

use clap::{App, Arg};

type WCResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
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

    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
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
    println!("{:?}", config);
    Ok(())
}
