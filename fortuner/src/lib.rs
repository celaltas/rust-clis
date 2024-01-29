use std::{
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::{App, Arg};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let file_args = Arg::with_name("files")
        .value_name("FILE")
        .help("Input files or directories")
        .multiple(true)
        .required(true);

    let pattern_arg = Arg::with_name("pattern")
        .short("m")
        .long("pattern")
        .takes_value(true)
        .value_name("PATTERN")
        .help("Pattern");
    let seed_arg = Arg::with_name("seed")
        .short("s")
        .long("seed")
        .takes_value(true)
        .value_name("SEED")
        .help("Random seed");

    let insensitive_arg = Arg::with_name("insensitive")
        .short("i")
        .long("insensitive")
        .help("Case-insensitive");

    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust fortune")
        .arg(pattern_arg)
        .arg(seed_arg)
        .arg(insensitive_arg)
        .arg(file_args)
        .get_matches();

    let pattern = matches
        .value_of("pattern")
        .map(|p| {
            RegexBuilder::new(p)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_e| format!("Invalid pattern \"{}\"", p))
        })
        .transpose()?;

    let seed = matches
        .value_of("seed")
        .map(|s| {
            s.parse::<u64>()
                .map_err(|_e| format!("\"{}\" not a valid integer", s))
        })
        .transpose()?;

    Ok(Config {
        sources: matches.values_of_lossy("files").unwrap(),
        seed: seed,
        pattern: pattern,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;

    if let Some(pattern) = config.pattern {
        let mut prev_source = None;
        for fortune in fortunes
            .iter()
            .filter(|fortune| pattern.is_match(&fortune.text))
        {
            if prev_source.as_ref().map_or(true, |s| s != &fortune.source) {
                eprintln!("({})\n%", fortune.source);
                prev_source = Some(fortune.source.clone());
            }
            println!("{}\n%", fortune.text);
        }
    } else {
        println!(
            "{}",
            pick_fortune(&fortunes, config.seed).unwrap_or_else(|| "No fortunes found".to_string())
        )
    }

    Ok(())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut all_files = vec![];
    for path in paths {
        match fs::metadata(path) {
            Ok(_) => {
                let entries: Vec<PathBuf> = WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                    .map(|entry| entry.path().into())
                    .collect();
                all_files.extend(entries)
            }
            Err(e) => return Err(format!("{}: {}", path, e).into()),
        }
    }
    all_files.sort();
    all_files.dedup();
    Ok(all_files)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = vec![];
    let mut buf = vec![];

    for path in paths {
        let file = BufReader::new(
            File::open(path)
                .map_err(|e| format!("{}: {}", path.to_string_lossy().into_owned(), e))?,
        );

        for line in file.lines().filter_map(Result::ok) {
            if line == "%" {
                if !buf.is_empty() {
                    let text = buf.join("\n");
                    fortunes.push(Fortune {
                        source: path.file_name().unwrap().to_string_lossy().into_owned(),
                        text: text,
                    });
                }
                buf.clear();
            } else {
                buf.push(line)
            }
        }
    }

    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    if let Some(val) = seed {
        let mut rng = StdRng::seed_from_u64(val);
        fortunes
            .choose(&mut rng)
            .map(|fortune| fortune.text.clone())
    } else {
        let mut rng = rand::thread_rng();
        fortunes
            .choose(&mut rng)
            .map(|fortune| fortune.text.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{find_files, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

    #[test]
    fn test_pick_fortune() {
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                    attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }

    #[test]
    fn test_read_fortunes() {
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
            A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
            A: A bad idea (bad-eye deer)."
            );
        }

        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_find_files() {
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
}
