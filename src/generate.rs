extern crate radix_trie;

use std::collections::hash_set::HashSet;

#[derive(PartialEq, Eq, Debug)]
pub struct BytesTrieKey(Vec<u8>);

impl ::radix_trie::TrieKey for BytesTrieKey {
    fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

pub type Trie = ::radix_trie::Trie<BytesTrieKey, Option<usize>>;

fn main_result() -> ::std::result::Result<(), Box<::std::error::Error>> {
    use std::io::{BufRead};

    let args : Vec<String> = ::std::env::args().collect();
    if args.len() != 2 {
        println!("usage: {} REDUCED_WORDLIST_FILE", args[0]);
        return Ok(());
    }

    let mut trie = Trie::new();
    let mut word_set = HashSet::<Vec<u8>>::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[1]))).split('\n' as u8) {
        let word = try!(maybe_word);
        word_set.insert(word.clone());
        trie.insert(BytesTrieKey(word), None);
    }
    assert_eq!(trie.len(), word_set.len());
    println!("word count: {}", word_set.len());


    let mut overlap_upper_bound = 16; // differentiation has 15 letters

    // Let's just find the biggest overlap.
    'outer: loop {
        let mut most_overlap = 0;
        let mut best_word = Vec::new();
        'inner: for word in &word_set {
            let start = if word.len() > overlap_upper_bound { word.len() - overlap_upper_bound } else { 1 };
            for idx in start..word.len() {
                let key = BytesTrieKey(word[idx..].to_vec());
                match trie.get_descendant(&key) {
                    Some(node) if node.len() > 0 => {
                        let overlap_len = word.len() - idx;
                        if overlap_len > most_overlap {
                            'find_word: for key in node.keys() {
                                // Don't form a cycle!
                                if &key.0 != word {
                                    most_overlap = overlap_len;
                                    best_word = word.clone();
                                    if most_overlap == overlap_upper_bound {
                                        break 'inner;
                                    } else {
                                        break 'find_word;
                                    }
                                } /*  else {
                                    panic!("Would have formed a cycle: {:?}", ::std::str::from_utf8(word));
                                } */
                            }

                        }
                    }
                    _ => {}
                }
            }
        }

        println!("most_overlap = {}", most_overlap);
        if most_overlap == 0 {
            break 'outer;
        }

        word_set.remove(&best_word);
        trie.remove(&BytesTrieKey(best_word.clone()));

        let overlap = &best_word[(best_word.len() - most_overlap)..];

        let key = BytesTrieKey(overlap.to_vec());
        let to_remove_key = BytesTrieKey(
            trie.get_descendant(&key).expect("broken trie?").keys().next().expect("no key?").0.clone());

        trie.remove(&to_remove_key);
        let trie_word = to_remove_key.0;

        word_set.remove(&trie_word);

        let mut new_word = best_word.clone();
        for idx in most_overlap .. trie_word.len() {
            new_word.push(trie_word[idx]);
        }

        println!("new_word = {:?}", ::std::str::from_utf8(&new_word));
        word_set.insert(new_word.clone());
        let new_key = BytesTrieKey(new_word);
        trie.insert(new_key, None);

        overlap_upper_bound = most_overlap;

        assert_eq!(word_set.len(), trie.len());
        println!("particle count: {}", word_set.len());
    }


    println!("OUTPUT -----");
    for word in word_set {
        println!("{}", ::std::str::from_utf8(&word).unwrap());
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
