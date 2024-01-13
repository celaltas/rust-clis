use clap::{App, Arg};

fn main() {
    let text_arg = Arg::with_name("text")
        .value_name("TEXT")
        .help("input text")
        .required(true)
        .min_values(1);
    let omit_newline_arg = Arg::with_name("omit_newline")
        .short("n")
        .help("Do not print newline")
        .takes_value(false);

    let matches = App::new("echor")
        .version("0.1.0")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust Echo")
        .arg(text_arg)
        .arg(omit_newline_arg)
        .get_matches();

    let text = matches.values_of_lossy("text").unwrap();
    let omit_newline = matches.is_present("omit_newline");

    let ending = if omit_newline { "" } else { "\n" };
    print!("{}{}", text.join(" "), ending);
}
