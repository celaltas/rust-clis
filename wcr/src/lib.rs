use std::error::Error;



type WCResult<T> = Result<T, Box<dyn Error>>;


#[derive(Debug)]
pub struct Config{
    files:Vec<String>,
    lines:bool,
    words:bool,
    bytes:bool,
    chars:bool,
}


pub fn get_args()->WCResult<Config>{
    Ok(Config{
        files:vec!["test".to_string()],
        lines:true,
        words:true,
        bytes:true,
        chars:true,
    })
}

pub fn run(config:Config)->WCResult<()>{
    println!("{:?}", config);
    Ok(())
}