use bigram::{run, Config};
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Error parsing: {}", err);
        process::exit(9);
    });
    println!("Generating bigram histogram for {}", config.get_file_name());
    run(config).unwrap();
}
