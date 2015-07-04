extern crate carrycoat;


pub fn main_result() -> ::std::result::Result<(), Box<::std::error::Error>> {
    use std::io::{BufRead, Read};

    let args : Vec<String> = ::std::env::args().collect();
    if args.len() != 4 {
        println!("usage: {} PORTMANTOUT_FILE WORDLIST_FILE NORMALIZED_WORDLIST_FILE", args[0]);
        return Ok(());
    }

    let mut portmantout = Vec::new();
    try!(try!(::std::fs::File::open(&args[1])).read_to_end(&mut portmantout));

    let mut words = ::std::collections::hash_set::HashSet::<Vec<u8>>::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[2]))).split('\n' as u8) {
        words.insert(try!(maybe_word));
    }

    println!("word count: {}", words.len());

    let mut normalized_words = Vec::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[3]))).split('\n' as u8) {
        normalized_words.push(try!(maybe_word));
    }
    println!("normalized word count: {}", normalized_words.len());

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
