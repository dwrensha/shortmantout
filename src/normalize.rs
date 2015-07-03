

//! Receives an (ascii, newline-delimited) wordlist on stdin. Removes all words that are contained in
//! other words. Writes the resulting word list to stdout.
//!

pub fn contains_subsequence(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    'outer: for start in 0..(haystack.len() + 1 - needle.len()) {
        for idx in 0..needle.len() {
            if needle[idx] != haystack[start + idx] {
                continue 'outer;
            }
        }
        return true;
    }
    return false;
}

#[test]
fn test_subseq() {
    assert!(contains_subsequence(&[1,3,4], &[3,4]));
    assert!(!contains_subsequence(&[1,3,4], &[3,5]));
    assert!(!contains_subsequence(&[1,3,4], &[3,4,5]));
}

pub fn main() {
    use std::io::BufRead;

    println!("hello world");
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
            if contains_subsequence(other_word.as_bytes(), word.as_bytes()) {
                continue 'outer;
            }
        }
        normalized.push(word.to_string());
    }

    println!("number of words: {}", words.len());
    for word in &normalized {
        println!("{}", word);
    }
}
