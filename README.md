# Solving the bigram parsing problem using Rust

## Introduction:
This small command line application aims to count the number of bigrams in a given 
text file. For e.g. given the text: 
“The quick brown fox and the quick blue hare.” The bigrams with their counts would be.
*	“the quick” 2
*	“quick brown” 1
*	“brown fox” 1
*	“fox and” 1
*	“and the” 1
*	“quick blue” 1
*	“blue hare” 1

### Installation:
**Pre-requisites:** This app requires rust to be installed on the machine
where it is being built. As long as rust is installed the executable will be
generated seamlessly on a nix or windows OS. To install rust, please visit
https://www.rust-lang.org/tools/install

This app was built and tested using rust version 1.43.1

Installation is straightforward. Download or clone this repo and from within the rustbigram
directory run the following command
```shell script
cargo build --release
```
This will create a binary named bigram under the /target/release folder.
Assuming it is being built on a *nix OS, to run it against a text file use the 
below command (while still in the rustbigram directory):
```shell script
./target/release/bigram /path/to/file.txt
```

For e.g. to test against the sample test file provided in the repo, 
please use the below command:
```shell script
./target/release/bigram sampletests/test1.txt
```
This should produce an output similar to below:
```shell script
Generating bigram histogram for sampletests/test1.txt
•	"the quick" 2
•	"quick brown" 1
•	"brown fox" 1
•	"fox and" 1
•	"and the" 1
•	"quick blue" 1
•	"blue hare" 1
Total no. of bigrams generated: 7
```

### Testing:
Following the rust convention, the unit tests are written in the lib.rs 
file itself under the tests module. 
To run the unit tests please issue the following command:
```shell script
cargo test
```

There are also 4 sample test files provided under the sampletests directory.
* test1.txt: This file has a very basic single line of text
* test2.txt: This file has a single line littered with extra quotations
* test3.txt: This file has a linebreak along with extra quotations
* test4.txt: This file has a single line littered with extra quotations and non-ascii text

### Performance:
On a mac running OS version 10.14.5 and 16GB RAM it took little under 4 minutes
to parse a 200MB file generating 11k bigrams.

There is a potential to improve the code if we are not concerned by the ordering of
the bigram keys. Since a hashmap by default does not iterate over keys in order
therefore we are using a separate vector to keep track of when a key was added. This
vector can be removed if the ordering is not important, potentially increasing the performance.

### Limitations:
* The app only accepts valid UTF-8 files. Although both unix LF and windows CRLF
line breakers are supported
* The app discards any text that comes after a quotation mark by design.
For e.g. scott's will be treated as scott and her'cules will be treated as her
