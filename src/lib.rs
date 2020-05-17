use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut counter_map: HashMap<String, u32> = HashMap::new();

    let mut rolling_vector: Vec<String> = vec![];

    let mut key_tracker: Vec<String> = vec![];

    let lines = read_lines(&config.filename).unwrap_or_else(|err| {
        println!("Cannot read the file: {}", err);
        process::exit(9);
    });

    for (line_no, line) in lines.enumerate() {
        let line = line.unwrap_or_else(|err| {
            println!("Could not read line no {}: {}", line_no, err);
            process::exit(9);
        });

        let line_word_vec = parse_text_into_vec(&line);

        for text in line_word_vec.iter() {
            calculate_counts(
                &mut counter_map,
                &mut rolling_vector,
                text,
                &mut key_tracker,
            );
        }
    }

    for key in key_tracker {
        println!("{}:{}", key, counter_map.get(&key).unwrap());
    }

    Ok(())
}

fn get_regex() -> Regex {
    Regex::new(r"[^a-z0-9 ]+").unwrap()
}

fn cleanse_word<'a>(text: &'a str, re: &Regex) -> Option<&'a str> {
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

fn parse_text_into_vec(line: &str) -> Vec<String> {
    line.split_whitespace()
        .filter(|s| cleanse_word(s.to_ascii_lowercase().as_str(), &get_regex()).is_some())
        .map(|s| {
            cleanse_word(s.to_ascii_lowercase().as_str(), &get_regex())
                .unwrap()
                .to_string()
        })
        .collect()
}

#[allow(mutable_borrow_reservation_conflict)]
fn calculate_counts(
    counter_map: &mut HashMap<String, u32>,
    rolling_vector: &mut Vec<String>,
    word: &str,
    key_tracker: &mut Vec<String>,
) {
    if rolling_vector.len() < 2 {
        rolling_vector.push(word.to_string());
    }
    if rolling_vector.len() == 2 {
        let key = get_key_from_vec(&rolling_vector);
        if counter_map.contains_key(&key) {
            let count = counter_map.get(&key).unwrap();
            counter_map.insert(key, count + 1);
        } else {
            key_tracker.push(key.clone());
            counter_map.insert(key, 1);
        }
        //re-initialize the vector now with the second word
        *rolling_vector = vec![rolling_vector.get(1).unwrap().to_string()];
    }
}

fn get_key_from_vec(rolling_vector: &Vec<String>) -> String {
    let mut key = String::new();
    key.push_str(rolling_vector.get(0).unwrap());
    key.push_str(" ");
    key.push_str(rolling_vector.get(1).unwrap());
    key
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanse_word_with_no_punctuations() {
        let sample_text = "fox";
        assert_eq!(cleanse_word(&sample_text, &get_regex()), Some("fox"));
    }

    #[test]
    fn test_cleanse_word_with_punctuations_at_end() {
        let sample_text = "fox's";
        assert_eq!(cleanse_word(&sample_text, &get_regex()), Some("fox"));
    }

    #[test]
    fn test_cleanse_word_with_punctuations_at_start() {
        let sample_text = "...???...,,,,```fox";
        assert_eq!(cleanse_word(&sample_text, &get_regex()), Some("fox"));
    }

    #[test]
    fn test_cleanse_word_with_punctuations_both_ends() {
        let sample_text = "...???...,,,,```fox...!!!!!";
        assert_eq!(cleanse_word(&sample_text, &get_regex()), Some("fox"));
    }

    #[test]
    fn test_cleanse_word_with_all_punctuations() {
        let sample_text = "...???...,,,,```...!!!!!";
        assert_eq!(cleanse_word(&sample_text, &get_regex()), None);
    }

    #[test]
    fn test_cleanse_word_with_emojis() {
        let sample_text = "...???...,,,,```🥰😍fox...!!!!!";
        assert_eq!(cleanse_word(&sample_text, &get_regex()), Some("fox"));
    }

    #[test]
    fn test_get_key_from_vec() {
        let vec: Vec<String> = vec!["key1".to_string(), "key2".to_string()];
        assert_eq!(get_key_from_vec(&vec), String::from("key1 key2"));
    }

    #[test]
    #[should_panic]
    fn test_get_key_from_vec_with_bad_vector() {
        let vec: Vec<String> = vec![];
        get_key_from_vec(&vec);
    }

    #[test]
    fn test_calculate_counts() {
        let mut counter_map: HashMap<String, u32> = HashMap::new();
        let mut rolling_vector: Vec<String> = Vec::new();
        let mut key_tracker: Vec<String> = Vec::new();

        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "the",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "quick",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "brown",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "fox",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "and",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "the",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "quick",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "blue",
            &mut key_tracker,
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "hare",
            &mut key_tracker,
        );

        assert_eq!(*counter_map.get("the quick").unwrap(), 2 as u32);
        assert_eq!(*counter_map.get("quick blue").unwrap(), 1 as u32);
        assert_eq!(counter_map.contains_key("hare the"), false);

        assert_eq!(key_tracker.len(), 7);
        assert_eq!(key_tracker.get(0), Some(&"the quick".to_string()));
        assert_eq!(key_tracker.get(6), Some(&"blue hare".to_string()))
    }
}
