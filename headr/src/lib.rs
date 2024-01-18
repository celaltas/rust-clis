use std::error::Error;



type HeadResult<T>= Result<T, Box<dyn Error>>;

pub struct Config{}

pub fn get_args()->HeadResult<Config>{
    Ok(Config {})
}


pub fn run(config:Config)->HeadResult<()>{
    Ok(())
}