use std::{env, process}; 
use std::fs::read_to_string; 
fn main() {
    let args: Vec<String> = env::args().collect(); 
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}"); 
        process::exit(1); 
    }); 

    match read_to_string(config.file_path) {
        Ok(contents) => println!("File contents: \n{}", contents),
        Err(e) => {
            println!("Error reading the file: {e}"); 
            process::exit(1); 
        }
    }

}


struct Config {
    file_path: String, 
}
impl  Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments, add a file argument"); 
        }

        let file_path = args[1].clone(); 

        Ok(Config { file_path })
    }
}