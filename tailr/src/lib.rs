use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

static NUM_RE: OnceCell<Regex> = OnceCell::new();
type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let file_args = Arg::with_name("files")
        .value_name("FILE")
        .help("input file(s)")
        .required(true)
        .multiple(true);

    let line_arg = Arg::with_name("lines")
        .short("n")
        .long("lines")
        .help("Number of lines")
        .default_value("10")
        .takes_value(true);

    let byte_arg = Arg::with_name("bytes")
        .short("c")
        .long("bytes")
        .conflicts_with("lines")
        .help("Number of bytes")
        .takes_value(true);

    let quiet_arg = Arg::with_name("quiet")
        .short("q")
        .long("quiet")
        .help("Suppress headers")
        .takes_value(false);

    let matches = App::new("tailr")
        .version("0.1.0")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust tail")
        .arg(file_args)
        .arg(line_arg)
        .arg(byte_arg)
        .arg(quiet_arg)
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_num_without_regex)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;
    let bytes = matches
        .value_of("bytes")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        bytes: bytes,
        quiet: matches.is_present("quiet"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        match File::open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if !config.quiet && num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                let file = BufReader::new(file);
                if let Some(num_bytes) = &config.bytes {
                    print_bytes(file, num_bytes, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?;
                }
            }
        }
    }
    Ok(())
}

fn parse_num_without_regex(val: &str) -> MyResult<TakeValue> {
    let signs = &['+', '-'];
    let res = val
        .starts_with(signs)
        .then(|| val.parse())
        .unwrap_or_else(|| val.parse().map(i64::wrapping_neg));
    match res {
        Ok(num) => {
            if num == 0 && val.starts_with('+') {
                Ok(PlusZero)
            } else {
                Ok(TakeNum(num))
            }
        }
        Err(_) => Err(From::from(val)),
    }
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let num_re = NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap());
    match num_re.captures(val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("-", |f| f.as_str());
            let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
            if let Ok(val) = num.parse() {
                if sign == "+" && val == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(val))
                }
            } else {
                Err(From::from(val))
            }
        }
        None => Err(From::from(val)),
    }
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut buf = vec![];
    let mut num_lines = 0;
    let mut num_bytes = 0;
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }

    Ok((num_lines, num_bytes))
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let read_bytes = file.read_until(b'\n', &mut buf)?;
            if read_bytes == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf))
            }
            line_num += 1;
            buf.clear();
        }
    }
    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }

    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total > 0 {
                Some(0)
            } else {
                None
            }
        }
        TakeNum(num) => {
            if num == &0 || total == 0 || num > &total {
                None
            } else {
                let start = if num < &0 { total + num } else { num - 1 };
                Some(if start < 0 { 0 } else { start as u64 })
            }
        }
    }
}
#[cfg(test)]
mod tests {

    use super::{
        count_lines_bytes, get_start_index, parse_num, parse_num_without_regex, TakeValue::*,
    };

    #[test]
    fn test_get_start_index() {
        assert_eq!(get_start_index(&PlusZero, 0), None);
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));
        assert_eq!(get_start_index(&TakeNum(0), 1), None);
        assert_eq!(get_start_index(&TakeNum(1), 0), None);
        assert_eq!(get_start_index(&TakeNum(2), 1), None);
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));
        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_parse_num() {
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));
        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
    #[test]
    fn test_parse_num_without_regex() {
        let res = parse_num_without_regex("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        let res = parse_num_without_regex("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        let res = parse_num_without_regex("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        let res = parse_num_without_regex("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        let res = parse_num_without_regex("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        let res = parse_num_without_regex(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num_without_regex(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num_without_regex(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));
        let res = parse_num_without_regex(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        let res = parse_num_without_regex("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        let res = parse_num_without_regex("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
