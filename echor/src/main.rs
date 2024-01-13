use clap::App;

fn main() {
    let matches = App::new("echor")
        .version("0.1.0")
        .author("Celal Taş <celal.tas123@gmail.com>")
        .about("Rust Echo")
        .get_matches();
    println!("{:#?}", matches);
}
