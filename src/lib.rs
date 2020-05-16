use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;
use std::process;

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let re = Regex::new(r"[^a-z0-9 ]+").unwrap();

    let lines = read_lines(&config.filename).unwrap_or_else(|err| {
        println!("Cannot read the file: {}", err);
        process::exit(9);
    });

    for (line_no, line) in lines.enumerate() {
        let l = line.unwrap_or_else(|err| {
            println!("Could not read line no {}: {}", line_no, err);
            process::exit(9);
        });
        println!("Line read is \n {}\n", l);
        let x: Vec<String> = l
            .split_ascii_whitespace()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        for (index, text) in x.iter().enumerate() {
            println!(
                "Original word is -->{}<--\nCleansed word is {:?}\n",
                text,
                cleanse_word(text, &re)
            )
        }
    }
    Ok(())
}

fn cleanse_word<'a>(text: &'a str, re: &'a Regex) -> Option<&'a str> {
    if re.is_match(text) {
        let start_idx = re.find(text).unwrap().start();
        let end_idx = re.find(text).unwrap().end();

        if start_idx != 0 {
            //The punctuations come after the word, split and disregard the rest
            return Some(text.split_at(start_idx).0);
        }

        if start_idx == 0 && text.len() > end_idx {
            //Check for the case where there are punctuations at the start
            //It can have two sub-cases:
            //1. The punctuations surround the word on both sides
            //2. The punctuations are only at the beginning
            let temp = text.split_at(end_idx).1;

            if re.is_match(temp) {
                //if there is a match that means that the word is surrounded by punctuations
                //like ......harry...''''' and we have split the first part off so now we are
                //left with harry...'''''
                //We need to discard the trailing punctuations as done in the first case
                return Some(temp.split_at(re.find(temp).unwrap().start()).0);
            } else {
                //The punctuations are only at the beginning so we can return the split word
                return Some(temp);
            }
        }

        if start_idx == 0 && text.len() == end_idx {
            //Its all punctuations no need to do anything
            return None;
        }
    }
    //The word is clean already, return as is
    Some(text)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Not enough args");
        }
        Ok(Config {
            filename: args[1].clone(),
        })
    }
    pub fn get_file_name(&self) -> &str {
        self.filename.as_ref()
    }
}
