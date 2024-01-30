use ansi_term::Style;
use chrono::NaiveDate;
use chrono::{Datelike, Local};
use clap::{App, Arg};
use itertools::izip;
use std::error::Error;
use std::str::FromStr;

type MyResult<T> = Result<T, Box<dyn Error>>;

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
const LINE_WIDTH: usize = 22;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

pub fn get_args() -> MyResult<Config> {
    let year_arg = Arg::with_name("show_current_year")
        .value_name("SHOW_YEAR")
        .short("y")
        .long("year")
        .help("Show whole current year")
        .conflicts_with_all(&["month", "year"])
        .takes_value(false);
    let month_arg = Arg::with_name("month")
        .value_name("MONTH")
        .short("m")
        .help("Month name or number (1-12)")
        .takes_value(true);

    let year_arg_2 = Arg::with_name("year")
        .value_name("YEAR")
        .help("Year (1-9999)");

    let matches = App::new("calr")
        .version("0.1.0")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust cal")
        .arg(year_arg)
        .arg(month_arg)
        .arg(year_arg_2)
        .get_matches();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;
    let today = Local::today();
    if matches.is_present("show_current_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }
    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today: today.naive_local(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            println!("{:>32}", config.year);
            let months: Vec<_> = (1..=12)
                .into_iter()
                .map(|month| format_month(config.year, month, false, config.today))
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
    match parse_int(month) {
        Ok(num) => {
            if (1..=12).contains(&num) {
                Ok(num)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", month).into())
            }
        }
        Err(_) => {
            let lower = &month.to_lowercase();
            for (index, item) in MONTH_NAMES.iter().enumerate() {
                if item.to_lowercase().starts_with(lower) {
                    println!("index: {}", index);
                    return Ok((index + 1) as u32);
                }
            }
            Err(format!("Invalid month \"{}\"", month).into())
        }
    }
}
fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|num| {
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    match val.parse::<T>() {
        Ok(val) => Ok(val),
        Err(_) => Err(From::from(format!("Invalid integer \"{}\"", val))),
    }
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd(year, month, 1);
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
        .into_iter()
        .map(|_| " ".to_string())
        .collect();

    let is_today = |day: u32| year == today.year() && month == today.month() && day == today.day();
    let last = last_day_in_month(year, month);
    days.extend((first.day()..=last.day()).into_iter().map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));

    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
        "{:^20} ", // two trailing spaces
        if print_year {
            format!("{} {}", month_name, year)
        } else {
            month_name.to_string()
        }
    ));
    lines.push(
        "Su Mo Tu We Th Fr Sa
"
        .to_string(),
    ); // two trailing spaces
    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$} ", // two trailing spaces
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }
    while lines.len() < 8 {
        lines.push(" ".repeat(LINE_WIDTH));
    }
    lines
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or(NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDate};

    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(last_day_in_month(2020, 1), NaiveDate::from_ymd(2020, 1, 31));
        assert_eq!(last_day_in_month(2020, 2), NaiveDate::from_ymd(2020, 2, 29));
        assert_eq!(last_day_in_month(2020, 4), NaiveDate::from_ymd(2020, 4, 30));
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd(2020, 1, 1);
        let leap_february = vec![
            "    February 2020    ",
            "Su Mo Tu We Th Fr Sa ",
            "                  1  ",
            " 2  3 4  5  6  7  8  ",
            " 9 10 11 12 13 14 15 ",
            "16 17 18 19 20 21 22 ",
            "23 24 25 26 27 28 29 ",
            "                     ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);
    }
    #[test]
    fn test_parse_int() {
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

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
}
