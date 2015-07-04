extern crate radix_trie;
extern crate carrycoat;

#[derive(PartialEq, Eq, Debug)]
pub struct BytesTrieKey(Vec<u8>);

impl ::radix_trie::TrieKey for BytesTrieKey {
    fn encode(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

pub type Trie = ::radix_trie::Trie<BytesTrieKey, ()>;

fn verify_contains_all(portmantout: &[u8], words: &[Vec<u8>]) -> Result<(), Vec<u8>> {
    for word in words {
        if !::carrycoat::contains_subsequence(portmantout, &word) {
            return Err(word.clone());
        }
    }
    return Ok(());
}

fn verify_cover(portmantout: &[u8], words: &Trie) -> Result<(), usize> {
    let mut verified_thru :usize = 0;
    let mut word_start_idx :usize = 0;
    let mut word = Vec::new();
    let mut good_word;
    'outer: while verified_thru + 1 < portmantout.len() {
        if word_start_idx > verified_thru {
            return Err(verified_thru + 1);
        }

        good_word = None;
        word.clear();
        let mut word_end_idx = word_start_idx;

        while word_end_idx <= verified_thru + 1 {
            word.push(portmantout[word_end_idx]);
            word_end_idx += 1;
        }

        {
            let key = BytesTrieKey(word.clone());
            match words.get_descendant(&key) {
                Some(node) if node.len() > 0 => {
                    if words.get(&key).is_some() {
                        good_word = Some(word.clone());
                    }
                }
                _ => {
                    // we can't make a word starting with this letter!
                    word_start_idx += 1;
                    continue 'outer;
                }
            }
        }

        'inner: while word_end_idx < portmantout.len() {
            word.push(portmantout[word_end_idx]);
            // see whether we can add one more letter
            let key = BytesTrieKey(word.clone());
            match words.get_descendant(&key) {
                Some(node) if node.len() > 0 => {
                    if words.get(&key).is_some() {
                        good_word = Some(word.clone());
                    }
                    word_end_idx += 1;
                    continue 'inner;
                }
                _ => { }
            }
            // We can't add one more letter.
            word.pop();

            match good_word {
                Some(ref w) => {
                    verified_thru = word_start_idx + w.len() - 1;
                }
                None => {}
            }
            word_start_idx += 1;
            continue 'outer;
        }

        {
            let key = BytesTrieKey(word.clone());
            if words.get(&key).is_some() {
                return Ok(());
            } else {
                return Err(verified_thru + 1);
            }
        }
    }
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

    // Get rid of any trailing whitespace.
    while (portmantout[portmantout.len() - 1] as char).is_whitespace() {
        portmantout.pop();
    }

    println!("The candidate portmantout has {} characters.", portmantout.len());

    let mut words = Trie::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[2]))).split('\n' as u8) {
        words.insert(BytesTrieKey(try!(maybe_word)), ());
    }

    println!("word count: {}", words.len());

    match verify_cover(&portmantout, &words) {
        Ok(()) => {
            println!("success! there is a cover.");
        }
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
        Ok(()) => {
            println!("success! contains all words");
        }
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
