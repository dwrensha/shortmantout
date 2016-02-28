extern crate radix_trie;
extern crate rand;
extern crate byteorder;

use std::collections::HashMap;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(PartialEq, Eq, Debug)]
pub struct BytesTrieKey(Vec<u8>);

impl ::radix_trie::TrieKey for BytesTrieKey {
    fn encode(&self) -> Vec<u8> {
        self.0.clone()
    }
}

pub type Trie = ::radix_trie::Trie<BytesTrieKey, ()>;
pub type ParticleTrie = ::radix_trie::Trie<BytesTrieKey, usize>;

struct Edge {
    next_idx: usize,
    padding: Vec<u8>,
    overlap_word: Vec<u8>,
}

struct Prev {
    prev_idx: usize,
    chain_start_idx: usize,
}

struct Particle {
    chars: Vec<u8>,
    next: Option<Edge>,
    prev: Option<Prev>,
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
    unconnected_on_right: Vec<usize>,

   // Set of indices of base particles unconnected on the left.
    unconnected_on_left: Vec<usize>,

    starticle_idx: usize,
}

impl State {
    fn new() -> State {
        State {
            particles: Vec::new(),
            score: 0,
            unconnected_on_right: Vec::new(),
            unconnected_on_left: Vec::new(),
            starticle_idx: 0,
        }
    }

    fn add_starticle(&mut self, particle: Vec<u8>) {
        let particle = Particle::new(particle);
        self.score += particle.chars.len();
        self.particles.push(particle);
        let idx = self.particles.len() - 1;
        self.unconnected_on_right.push(idx);
        self.starticle_idx = idx;
    }

    fn add_particle(&mut self, particle: Vec<u8>) {
        let particle = Particle::new(particle);
        self.score += particle.chars.len();
        self.particles.push(particle);
        let idx = self.particles.len() - 1;
        self.unconnected_on_right.push(idx);
        self.unconnected_on_left.push(idx);
    }
}

fn coalesce<R>(state: &mut State, words_trie: &Trie, rng: &mut R) where R: rand::Rng {
    // first, form a trie containing all of the current chains of particles
    // then, while the length of unconnected_on_right is greater than 1:
    // pick a random idx in unconnected on right
    // connect that particle to one of the existing chains (but not itself!)
    //  (so temporarily remove self from the particle-chain trie?)
    //
    //

    let mut particles_trie = ParticleTrie::new();
    for &idx in &state.unconnected_on_left {
        let particle = &state.particles[idx];
        particles_trie.insert(BytesTrieKey(particle.chars.clone()), idx);
    }
    'outer: while particles_trie.len() > 0 {

        let mut best_padding: Option<Vec<u8>> = None;       // smaller is better
        let mut best_next_particle_idx: Option<usize> = None; // particle that corresponded to the best padding.
        let mut best_overlap_word: Option<Vec<u8>> = None;

        let idx = rng.gen_range(0, state.unconnected_on_right.len());
        let particle_idx = state.unconnected_on_right.swap_remove(idx);

        let chain_start_particle_idx = {
            let particle = &state.particles[particle_idx];
            match particle.prev {
                None => particle_idx,
                Some(ref prev) => prev.chain_start_idx,
            }
        };

        // Special case when we are almost done. We need to choose the starticle chain.
        if particles_trie.len() == 1 && chain_start_particle_idx != state.starticle_idx {
            state.unconnected_on_right.push(idx);
            continue;
        }

        {
            let particle = &state.particles[particle_idx];
            let chain_start_particle = &state.particles[chain_start_particle_idx];
            let chain_start_particle_trie_key = BytesTrieKey(chain_start_particle.chars.clone());
            // temporarily remove chain_start_particle from particles_trie, to avoid forming a cycle.
            if chain_start_particle_idx != state.starticle_idx {
                particles_trie.remove(&chain_start_particle_trie_key);
            }

            let start_idx = particle.chars.len() - ::std::cmp::min(11, particle.chars.len());

            'find_best: for suffix_start in start_idx .. particle.chars.len() {
                let suffix_len = particle.chars.len() - suffix_start;
                let suffix = particle.chars[suffix_start ..].to_vec();
                if let Some(node) = words_trie.get_descendant(&BytesTrieKey(suffix)) {
                    assert!(node.len() > 0);
                    // Cool. there is at least one word that starts with `suffix`.

                    for key in node.keys() {
                        let word = key.0.clone();
                        'added: for idx in suffix_len .. word.len() {
                            let padding_len = idx - suffix_len;
                            match best_padding {
                                Some(ref p) if p.len() < padding_len => {
                                    // We have no chance of doing better than our current best.
                                    break 'added;
                                }
                                _ => {}
                            }
                            match particles_trie.get_descendant(&BytesTrieKey(word[idx..].to_vec())) {
                                Some(particle_node) if particle_node.len() > 0 => {
                                    let &p_idx = particle_node.values().next().expect("no key?");
                                    best_padding = Some(word[suffix_len..idx].to_vec());
                                    best_next_particle_idx = Some(p_idx);
                                    best_overlap_word = Some(word.clone());
                                    if padding_len == 0 {
                                        // We're not going to do better than this.
                                        break 'find_best;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            if chain_start_particle_idx != state.starticle_idx {
                particles_trie.insert(chain_start_particle_trie_key, chain_start_particle_idx);
            }
        }
        if let (Some(next_particle_idx),
                Some(padding),
                Some(overlap_word)) = (best_next_particle_idx, best_padding, best_overlap_word) {
            state.score += padding.len();
//            state.unconnected_on_left.swap_remove(next_particle_idx);

            let chain_start_idx = {
                let particle = &mut state.particles[particle_idx];
                let edge = Edge {
                    next_idx: next_particle_idx,
                    overlap_word: overlap_word,
                    padding: padding,
                };
                particle.next = Some(edge);

                match particle.prev {
                    None => particle_idx,
                    Some(ref prev) => prev.chain_start_idx,
                }
            };


            {
                let next_particle = &mut state.particles[next_particle_idx];
                let prev = Prev {
                    prev_idx: particle_idx,
                    chain_start_idx: chain_start_idx,
                };
                next_particle.prev = Some(prev);
            }

            // propagate forward the changes to chain_start_idx.
            let mut idx = next_particle_idx;
            loop {
                let current_particle = &mut state.particles[idx];
                match current_particle.prev {
                    None => unreachable!(),
                    Some(ref mut prev) => {
                        prev.chain_start_idx = chain_start_idx
                    }
                }
                match current_particle.next {
                    Some(ref edge) => {
                        idx = edge.next_idx;
                    }
                    None => break,
                }
            }


            {
                let next_particle = &state.particles[next_particle_idx];
                particles_trie.remove(&BytesTrieKey(next_particle.chars.clone()));
            }
        } else {
            unreachable!()
        }
        println!("left: {}. score: {}", state.unconnected_on_right.len(), state.score);
    }

    println!("particles_trie.len(): {}", particles_trie.len());
    println!("unconnected_on_right: {:?}", state.unconnected_on_right);

    let mut portmantout = Vec::new();
    let mut current_idx = state.starticle_idx;
    let mut counter = 0;
    loop {
        counter += 1;
        if counter > 34000 {
            panic!("loopy!");
        }
        let particle = &state.particles[current_idx];
//        println!("particle chars: {:?}", ::std::str::from_utf8(&particle.chars));
        match particle.prev {
            None => {}
            Some(ref prev) => {
//                println!("chain start idx: {}", prev.chain_start_idx);
            }
        }

        for &c in &particle.chars {
            portmantout.push(c);
        }
        match particle.next {
            Some(ref edge) => {
                current_idx = edge.next_idx;
                for &c in &edge.padding {
                    portmantout.push(c);
                }
                //println!("overlap word: {:?}", ::std::str::from_utf8(&edge.overlap_word));
            }
            None => {
                break;
            }
        }
    }
    println!("counter: {}", counter);
    println!("portmanteau: \n\n\n {}", ::std::str::from_utf8(&portmantout[..]).unwrap());
}

fn main_result() -> ::std::result::Result<(), Box<::std::error::Error>> {
    use std::io::{BufRead};

    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 4 {
        println!("usage: {} PARTICLES_FILE JOINERS_FILE WORDLIST_FILE", args[0]);
        return Ok(());
    }

    let mut state = State::new();

    let mut found_starticle = false;
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[1]))).split('\n' as u8) {
        let word = try!(maybe_word);
        if !found_starticle && word.starts_with("portmanteau".as_bytes()) {
            found_starticle = true;
            state.add_starticle(word);
        } else {
            state.add_particle(word);
        }
    }

    println!("score: {}, starticle idx: {}", state.score, state.starticle_idx);

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


    let mut urandom = try!(::std::fs::File::open("/dev/urandom"));
    let mut seed: [u32; 4] = [0; 4];
    for idx in 0..4 {
        seed[idx] = try!(urandom.read_u32::<LittleEndian>());
    }
    println!("seed {:?}", seed);

    let mut rng: rand::XorShiftRng = rand::SeedableRng::from_seed(seed);

    coalesce(&mut state, &words_trie, &mut rng);

/*
    println!("words in word trie: {}", words_trie.len());

    let mut portmantout = Vec::new();

    let key = BytesTrieKey("portmanteau".as_bytes().to_vec());
    let starticle = particles_trie.get_descendant(&key).and_then(|node| {node.keys().next()})
        .expect("no particle starts with 'portmanteau'?").0.clone();


    for c in &starticle {
        portmantout.push(*c);
    }

    particles_trie.remove(&BytesTrieKey(starticle));


    println!("OUTPUT -----");
    println!("{}", ::std::str::from_utf8(&portmantout).unwrap());
*/
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
