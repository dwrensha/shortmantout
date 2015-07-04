extern crate carrycoat;

use std::collections::hash_set::HashSet;

fn verify_contains_all(portmantout: &[u8], words: &[Vec<u8>]) -> Result<(), Vec<u8>> {
    for word in words {
        if !::carrycoat::contains_subsequence(portmantout, &word) {
            return Err(word.clone());
        }
    }
    return Ok(());
}

fn verify_cover(portmantout: &[u8], words: &HashSet<Vec<u8>>) -> Result<(), usize> {
    return Ok(());
}

fn main_result() -> ::std::result::Result<(), Box<::std::error::Error>> {
    use std::io::{BufRead, Read};

    let args : Vec<String> = ::std::env::args().collect();
    if args.len() != 4 {
        println!("usage: {} PORTMANTOUT_FILE WORDLIST_FILE NORMALIZED_WORDLIST_FILE", args[0]);
        return Ok(());
    }

    let mut portmantout = Vec::new();
    try!(try!(::std::fs::File::open(&args[1])).read_to_end(&mut portmantout));

    let mut words = HashSet::<Vec<u8>>::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[2]))).split('\n' as u8) {
        words.insert(try!(maybe_word));
    }

    println!("word count: {}", words.len());

    match verify_cover(&portmantout, &words) {
        Ok(()) => {}
        Err(n) => {
            println!("fails to cover character at index {}", n);
        }
    }

    let mut normalized_words = Vec::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[3]))).split('\n' as u8) {
        normalized_words.push(try!(maybe_word));
    }
    println!("normalized word count: {}", normalized_words.len());

    match verify_contains_all(&portmantout, &normalized_words) {
        Ok(()) => {}
        Err(word) => {
            println!("does not contain {:?}", ::std::str::from_utf8(&word));
        }
    }

    return Ok(());
}

pub fn main() {
    match main_result() {
        Ok(()) => {}
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
