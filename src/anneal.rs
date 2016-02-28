extern crate radix_trie;
extern crate rand;

use std::collections::hash_map::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub struct BytesTrieKey(Vec<u8>);

impl ::radix_trie::TrieKey for BytesTrieKey {
    fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

pub type Trie = ::radix_trie::Trie<BytesTrieKey, ()>;

struct Edge {
    overlap_word: Vec<u8>,

}


struct Particle {
    chars: Vec<u8>,
    next: Option<Edge>,
    prev: Option<usize>, // index to
}

impl Particle {
    fn new(chars: Vec<u8>) -> Particle {
        Particle {
            chars: chars,
            next: None,
            prev: None,
        }
    }
}

struct State {
    pub particles: Vec<Particle>,
    score: usize,

   // Set of indices of base particles unconnected on the right.

   // Set of indices of base particles unconnected on the left.
}

impl State {
    fn add_particle(&mut self, particle: Particle) {
        self.score += particle.chars.len();
        self.particles.push(particle);
    }
}

fn main_result() -> ::std::result::Result<(), Box<::std::error::Error>> {
    use std::io::{BufRead};

    let args : Vec<String> = ::std::env::args().collect();
    if args.len() != 4 {
        println!("usage: {} PARTICLES_FILE JOINERS_FILE WORDLIST_FILE", args[0]);
        return Ok(());
    }


    let state = State {
        particles: Vec::new(),
        score: 0,
    };

    let mut particles = Vec::new();
    let mut particles_trie = Trie::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[1]))).split('\n' as u8) {
        let word = try!(maybe_word);
        Particle::new(word.clone());
        particles.push(word.clone());
        particles_trie.insert(BytesTrieKey(word), ());
    }

    let mut words_trie = Trie::new();

    let mut joiners = HashMap::<(u8, u8), Vec<u8>>::new();
    for maybe_joiner in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[2]))).split('\n' as u8) {
        let joiner = try!(maybe_joiner);
        words_trie.insert(BytesTrieKey(joiner.clone()), ());
        let key = (*joiner.first().unwrap(), *joiner.last().unwrap());
        if !joiners.contains_key(&key) {
            joiners.insert(key, joiner);
        }
    }

    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[3]))).split('\n' as u8) {
        let word = try!(maybe_word);
        if word.len() < 11 { // (optimization)
            words_trie.insert(BytesTrieKey(word), ());
        }
    }


    println!("words in word trie: {}", words_trie.len());

    let mut portmantout = Vec::new();

    let key = BytesTrieKey("portmanteau".as_bytes().to_vec());
    let starticle = particles_trie.get_descendant(&key).and_then(|node| {node.keys().next()})
        .expect("no particle starts with 'portmanteau'?").0.clone();


    for c in &starticle {
        portmantout.push(*c);
    }

    particles_trie.remove(&BytesTrieKey(starticle));

    'outer: while particles_trie.len() > 0 {
        let mut best_padding: Option<Vec<u8>> = None;
        let mut best_next_particle: Option<Vec<u8>> = None;
        let mut overlap_word: Option<Vec<u8>> = None;
        'find_best: for suffix_start in (portmantout.len() - 11)..(portmantout.len()) {
            best_padding = None;
            best_next_particle = None;
            let suffix_len = portmantout.len() - suffix_start;
            let suffix = portmantout[suffix_start ..].to_vec();
            if let Some(node) = words_trie.get_descendant(&BytesTrieKey(suffix)) {
                assert!(node.len() > 0);
                for key in node.keys() {
                    let word = key.0.clone();
                    'added: for idx in suffix_len .. word.len() {
                        let padding_len = idx - suffix_len;
                        match best_padding {
                            Some(ref p) if p.len() < padding_len => {
                                break 'added;
                            }
                            _ => {}
                        }
                        match particles_trie.get_descendant(&BytesTrieKey(word[idx..].to_vec())) {
                            Some(particle_node) if particle_node.len() > 0 => {
                                let p = particle_node.keys().next().expect("no key?").0.clone();
                                best_padding = Some(word[suffix_len..idx].to_vec());
                                best_next_particle = Some(p);
                                overlap_word = Some(word.clone());
                                if padding_len == 0 {
                                    break 'find_best;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        if let (Some(particle), Some(padding)) = (best_next_particle, best_padding) {
            println!("next: {:?}, {:?}, {:?}",
                     ::std::str::from_utf8(&overlap_word.unwrap()),
                     ::std::str::from_utf8(&padding),
                     ::std::str::from_utf8(&particle));
            for idx in 0..padding.len() {
                portmantout.push(padding[idx]);
            }
            for idx in 0..particle.len() {
                portmantout.push(particle[idx]);
            }

            particles_trie.remove(&BytesTrieKey(particle));
            println!("trie len: {}", particles_trie.len());
        } else {
            unreachable!()
        }
    }

    println!("OUTPUT -----");
    println!("{}", ::std::str::from_utf8(&portmantout).unwrap());

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
