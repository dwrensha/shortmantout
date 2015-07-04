//! Receives an (ascii, newline-delimited) wordlist on stdin. Removes all words that are contained in
//! other words. Writes the resulting word list to stdout.
//!

extern crate carrycoat;

pub fn main() {
    use std::io::BufRead;

    let stdin = ::std::io::stdin();
    let input = stdin.lock();

    let mut words = Vec::new();
    for line in input.lines() {
        words.push(line.unwrap());
    }

    let mut normalized = Vec::new();
    'outer: for word in &words {
        for other_word in &words {
            if word == other_word {
                continue;
            }
            if ::carrycoat::contains_subsequence(other_word.as_bytes(), word.as_bytes()) {
                continue 'outer;
            }
        }
        normalized.push(word.to_string());
    }

    for word in &normalized {
        println!("{}", word);
    }
}
