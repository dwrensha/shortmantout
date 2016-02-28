extern crate radix_trie;
extern crate rand;

use std::collections::HashMap;

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
        let idx = self.particles.len();
        self.unconnected_on_right.push(idx);
        self.starticle_idx = idx;
    }

    fn add_particle(&mut self, particle: Vec<u8>) {
        let particle = Particle::new(particle);
        self.score += particle.chars.len();
        self.particles.push(particle);
        let idx = self.particles.len();
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

    'outer: while state.unconnected_on_right.len() > 1 {

        let idx = rng.gen_range(0, state.unconnected_on_right.len());
        let particle_idx = state.unconnected_on_right.swap_remove(idx);
        let particle = &state.particles[particle_idx];

        let mut best_padding: Option<Vec<u8>> = None;       // smaller is better
        let mut best_next_particle_idx: Option<usize> = None; // particle that corresponded to the best padding.
        let mut overlap_word: Option<Vec<u8>> = None;

        let start_idx = ::std::cmp::max(0, particle.chars.len() - 11);

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
//                                let p_idx = particle_node.keys().next().expect("no key?").0;
                                best_padding = Some(word[suffix_len..idx].to_vec());
//                                best_next_particle_idx = Some(p_idx);
                                overlap_word = Some(word.clone());
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
/*
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
             */
    }

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


    let mut rng: rand::XorShiftRng = rand::SeedableRng::from_seed([1,2,3,4]);

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
