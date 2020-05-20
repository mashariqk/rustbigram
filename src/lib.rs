/*!
This crate provides a library for generating a histogram of bigrams contained within a text file.
The file has to be a valid UTF-8 format. Only ascii words are considered and trailing characters
after a punctuation are dropped.
 */
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;


/// Takes as an input a config object which contains path to a file. Runs through the file to
/// generate a histogram of bigrams contained in the file and prints them out.
/// Step 1: Initializes mutable objects counter_map, rolling_vector, key_tracker that will be used
/// to keep track of the bigrams and their counts
/// Step 2: Uses the read_lines function to get an iterable of lines of the file instead of loading
/// all the contents in a String object
/// Step 3: Iterates over the lines and passes each line to parse_text_into_vec function which
/// returns a vector of the words contained in the line disregarding any spaces or punctuations
/// Step 4: Iterates over the vector returned in the previous step and passes each element into the
/// calculate_counts function along with the objects initialized in step 1 to update the bigram counts
/// Step 5: Uses the key_tracker vector to iterate over the counter_map in order and writes the
/// histogram to the output console
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {

    let mut counter_map: HashMap<String, u32> = HashMap::new();

    let mut rolling_vector: Vec<String> = vec![];

    let re = get_regex();

    let lines = read_lines(&config.filename).unwrap_or_else(|err| {
        println!("Cannot read the file: {}", err);
        process::exit(9);
    });

    for (line_no, line) in lines.enumerate() {

        let line = line.unwrap_or_else(|err| {
            println!("Could not read line no {}: {}", line_no, err);
            process::exit(9);
        });

        let line_word_vec = parse_text_into_vec(&line,&re);

        for text in line_word_vec.iter() {
            calculate_counts(
                &mut counter_map,
                &mut rolling_vector,
                text
            );
        }
    }

    for (k,v) in counter_map.iter() {
        println!("•\t\"{}\" {}", k, v);
    }

    println!(
        "Total no. of bigrams generated: {}",
        counter_map.keys().len()
    );

    Ok(())
}

fn get_regex() -> Regex {
    Regex::new(r"[^a-z0-9 ]+").unwrap()
}

/// Takes as an input a string slice and splits it into a vector by splitting on white space
/// as well as manipulating the rendered elements through the cleanse_word function.
fn parse_text_into_vec(line: &str, re:&Regex) -> Vec<String> {
    line.split_whitespace()
        .filter(|s| cleanse_word(s.to_ascii_lowercase().as_str(), re).is_some())
        .map(|s| {
            cleanse_word(s.to_ascii_lowercase().as_str(), re)
                .unwrap()
                .to_string()
        })
        .collect()
}

/// Takes as input a string slice and a compiled regex pattern and returns an optional containing
/// either a string slice.
/// 1. If the regex matches any character of the string at an index > 0 then it strips away the
/// string from there and returns the leftover wrapped in a Some()
/// 2. If the regex matches at the start of the string then it strips away at the start until it
/// finds a non matching character. If there are no matches after that then it returns the slice.
/// 3. If the regex matches at the start of the string then it strips away at the start until it
/// finds a non matching character. If there is again a match then it splits the string from that
/// point and returns the slice.
/// 4. If everything matches the regex then it returns a None
/// 5. If nothing matches the regex it returns the original slice in a Some()
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

/// Takes as an input a mutable reference to a HashMap<String,u32>, a mutable reference to
/// two Vec<String> and a String slice. One vector (rolling_vector) is used to keep track of
/// bigram keys and it rolls over with each new bigram. The other vector (key_tracker) keeps track
/// of the sequence of bigrams as they were found in the text file. Since by default the iteration
/// of a Map is indeterminate, therefore we will lose track of the sequence of bigrams without
/// this vector. The string slice is the next word in the stream of words from the file, the
/// function decides whether to increase the count of an already existing bigram or to add this
/// word to another previous word to create a new bigram
#[allow(mutable_borrow_reservation_conflict)]
fn calculate_counts(
    counter_map: &mut HashMap<String, u32>,
    rolling_vector: &mut Vec<String>,
    word: &str
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
            counter_map.insert(key, 1);
        }
        //re-initialize the vector now with the second word
        *rolling_vector = vec![rolling_vector.get(1).unwrap().to_string()];
    }
}

/// Takes a vector and generates a string object from its two elements.
/// This function assumes that the vector does indeed have two elements in it.
fn get_key_from_vec(rolling_vector: &Vec<String>) -> String {
    let mut key = String::new();
    key.push_str(rolling_vector.get(0).unwrap());
    key.push_str(" ");
    key.push_str(rolling_vector.get(1).unwrap());
    key
}

/// Gives a Result object containing an iterable over lines of a file.
/// Much better to use this approach when dealing with a large file than putting all the
/// contents in a String object
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
    fn test_parse_text_into_vec_with_no_punctuations_and_all_uppercase() {
        let line = "THE QUICK BROWN FOX AND THE QUICK BROWN HARE";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_no_punctuations_and_mixedcase() {
        let line = "THE quICK brOWn FOX AND ThE QuiCK BROWN haRE";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_no_punctuations_and_mixedcase_and_extra_spaces() {
        let line = "THE quICK brOWn             FOX AND      ThE             QuiCK BROWN haRE";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.get(8), Some(&"hare".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_punctuations_at_end_and_mixedcase_and_extra_spaces() {
        let line = "THE quICK's brOWn'ss             FOX...??? AND      ThE             QuiCK BROWN haRE'ssssss";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.get(8), Some(&"hare".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_punctuations_at_start_and_mixedcase_and_extra_spaces() {
        let line =
            "THE .......quICK brOWn         FOX AND      ThE             QuiCK BROWN \"\"\"haRE";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.get(8), Some(&"hare".to_string()));
        assert_eq!(v.get(1), Some(&"quick".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_enclosing_punctuations_and_mixedcase_and_extra_spaces() {
        let line =
            "THE .......quICK!!!!! .....brOWn'ssss         FOX AND      ThE             QuiCK BROWN \"\"\"haRE\"\".....??????";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.get(8), Some(&"hare".to_string()));
        assert_eq!(v.get(1), Some(&"quick".to_string()));
        assert_eq!(v.get(2), Some(&"brown".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_enclosing_punctuations_and_mixedcase_and_extra_spaces_and_non_ascii(
    ) {
        let line =
            "THE ૱﷼₢quICK₱€₴ brOWn🤯🤯🤯         FOX AND      ThE             QuiCK BROWN \"\"\"🥰🥰🥰haRE\"\"..😍😍😍...??????";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.get(8), Some(&"hare".to_string()));
        assert_eq!(v.get(1), Some(&"quick".to_string()));
        assert_eq!(v.get(2), Some(&"brown".to_string()));
        assert_eq!(v.len(), 9);
    }

    #[test]
    fn test_parse_text_into_vec_with_all_punctuations_and_mixedcase_and_extra_spaces_and_non_ascii()
    {
        let line =
            "THE ૱﷼₢quICK₱€₴ brOWn🤯🤯🤯         FOX AND    ???...;;;;   ThE    ₱€₴૱﷼₢;;../////         QuiCK BROWN \"\"\"🥰🥰🥰haRE\"\"..😍😍😍...??????";
        let v = parse_text_into_vec(line,&get_regex());
        assert!(v.contains(&"quick".to_string()));
        assert!(!v.contains(&"THE".to_string()));
        assert_eq!(v.get(8), Some(&"hare".to_string()));
        assert_eq!(v.get(1), Some(&"quick".to_string()));
        assert_eq!(v.get(2), Some(&"brown".to_string()));
        assert_eq!(v.len(), 9);
    }

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

        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "the"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "quick"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "brown"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "fox"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "and"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "the"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "quick"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "blue"
        );
        calculate_counts(
            &mut counter_map,
            &mut rolling_vector,
            "hare"
        );

        assert_eq!(*counter_map.get("the quick").unwrap(), 2 as u32);
        assert_eq!(*counter_map.get("quick blue").unwrap(), 1 as u32);
        assert_eq!(counter_map.contains_key("hare the"), false);
    }
}
