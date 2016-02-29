extern crate radix_trie;
extern crate rand;
extern crate byteorder;

use std::collections::{HashMap, HashSet};

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

#[derive(Clone)]
enum Edge {
    Overlapped(usize),
    Padded { padding: Vec<u8>, overlap_word: Vec<u8> },
}

impl Edge {
    fn score(&self) -> isize {
        match *self {
            Edge::Overlapped(n) => -(n as isize),
            Edge::Padded { ref padding, .. } => padding.len() as isize,
        }
    }
}

#[derive(Clone)]
struct Next {
    next_idx: usize,
    edge: Edge,
}

#[derive(Clone, Debug)]
struct NoNext {
    chain_start_idx: usize,
}

#[derive(Clone)]
struct Prev {
    prev_idx: usize,
}

#[derive(Clone)]
struct NoPrev {
    chain_end_idx: usize,
}

#[derive(Clone)]
struct Particle {
    chars: Vec<u8>,
    next: Result<Next, NoNext>,
    prev: Result<Prev, NoPrev>,
}

impl Particle {
    fn new(chars: Vec<u8>, idx: usize) -> Particle {
        Particle {
            chars: chars,
            next: Err(NoNext {chain_start_idx: idx}),
            prev: Err(NoPrev {chain_end_idx: idx}),
        }
    }
}

#[derive(Clone)]
struct State {
    pub particles: Vec<Particle>,
    score: isize,

    // Set of indices of base particles unconnected on the right.
    unconnected_on_right: Vec<usize>,

   // Set of indices of base particles unconnected on the left.
    unconnected_on_left: HashSet<usize>,

    starticle_idx: usize,
}

impl State {
    fn new() -> State {
        State {
            particles: Vec::new(),
            score: 0,
            unconnected_on_right: Vec::new(),
            unconnected_on_left: HashSet::new(),
            starticle_idx: 0,
        }
    }

    fn add_starticle(&mut self, particle: Vec<u8>) {
        let idx = self.particles.len();
        let particle = Particle::new(particle, idx);
        self.score += particle.chars.len() as isize;
        self.particles.push(particle);
        self.unconnected_on_right.push(idx);
        self.starticle_idx = idx;
    }

    fn add_particle(&mut self, particle: Vec<u8>) {
        let idx = self.particles.len();
        let particle = Particle::new(particle, idx);
        self.score += particle.chars.len() as isize;
        self.particles.push(particle);
        self.unconnected_on_right.push(idx);
        self.unconnected_on_left.insert(idx);
    }

    fn sanity_check(&self) {
        assert_eq!(self.unconnected_on_right.len(), self.unconnected_on_left.len() + 1);
    }
}

fn break_chains<R>(state: &mut State, rng: &mut R) where R: rand::Rng {
    for particle_idx in 0..state.particles.len() {
        state.sanity_check();
        let maybe_next_idx = {
            let particle = &mut state.particles[particle_idx];
            if let Ok(ref next) = particle.next {
                if rng.gen_range(0, 10000) < 7 {
                    // We're going to break this up.
                    state.score -= next.edge.score();
                    Some(next.next_idx)
                } else {
                    None
                }
            } else {
                None
            }
        };


        if let Some(next_idx) = maybe_next_idx {

            state.unconnected_on_right.push(particle_idx);
            state.unconnected_on_left.insert(next_idx);

            // need to figure out the start and the end of the chain.
            // so walk forward to the end.
            let (chain_start_idx, chain_end_idx) = {
                let mut current_idx = next_idx;
                let chain_start_idx;
                loop {
                    let particle = &state.particles[current_idx];
                    match particle.next {
                        Ok(ref next) => {
                            current_idx = next.next_idx;
                        }
                        Err(ref no_next) => {
                            chain_start_idx = no_next.chain_start_idx;
                            break;
                        }
                    }
                }
                (chain_start_idx, current_idx)
            };

            {
                let chain_start = &mut state.particles[chain_start_idx];
                match chain_start.prev {
                    Ok(_) => unreachable!(),
                    Err(ref mut no_prev) => {
                        no_prev.chain_end_idx = particle_idx;
                    }
                }
            }

            {
                let chain_end = &mut state.particles[chain_end_idx];
                match chain_end.next {
                    Ok(_) => unreachable!(),
                    Err(ref mut no_next) => {
                        no_next.chain_start_idx = next_idx;
                    }
                }
            }


            state.particles[next_idx].prev = Err(NoPrev {chain_end_idx: chain_end_idx});
            state.particles[particle_idx].next = Err(NoNext {chain_start_idx: chain_start_idx});

        }
    }
}


fn write_portmantout(state: &State) -> Result<(), ::std::io::Error> {
    use std::io::Write;

    assert!(state.unconnected_on_left.len() == 0);
    assert!(state.unconnected_on_right.len() == 1);
    let filename = format!("out/{}.txt", state.score);
    let mut file = try!(::std::fs::File::create(&filename));
    let mut current_idx = state.starticle_idx;
    let mut counter = 0;
    loop {
        counter += 1;
        if counter > 150000 {
            panic!("loopy!");
        }
        let particle = &state.particles[current_idx];

        match particle.next {
            Ok(ref next) => {
                current_idx = next.next_idx;
                match next.edge {
                    Edge::Padded { ref padding, .. } => {
                        try!(file.write_all(&particle.chars));
                        try!(file.write_all(padding));
                    }
                    Edge::Overlapped(ref n) => {
                        let n = *n;
                        assert!(particle.chars.len() >= n);
                        let write_len = particle.chars.len() - n;
                        try!(file.write_all(&particle.chars[.. write_len]));
                    }
                }

                //println!("overlap word: {:?}", ::std::str::from_utf8(&edge.overlap_word));
            }
            Err(_) => {
                try!(file.write_all(&particle.chars));
                break;
            }
        }
    }
    Ok(())
}

fn find_next(state: &State, words_trie: &Trie,
             particle: &Particle, particles_trie: &ParticleTrie) -> Next {
    // first try for an overlapped edge.
    for start_idx in (particle.chars.len() - 3) .. particle.chars.len() {
        let overlap = &particle.chars[start_idx..];
        if let Some(node) = particles_trie.get_descendant(&BytesTrieKey(overlap.to_vec())) {
            assert!(node.len() > 0);
            let (_, &p_idx) = node.iter().next().expect("no value?");
            return Next {
                next_idx: p_idx,
                edge: Edge::Overlapped(overlap.len()),
            };
        }
    }

    // okay, we can't get any overlap.

    let mut best_padding: Option<Vec<u8>> = None;       // smaller is better
    let mut best_next_particle_idx: Option<usize> = None; // particle that corresponded to the best padding.
    let mut best_overlap_word: Option<Vec<u8>> = None;

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
                        Some(ref p) if p.len() <= padding_len => {
                            // We have no chance of doing better than our current best.
                            break 'added;
                        }
                        _ => {}
                    }
                    match particles_trie.get_descendant(&BytesTrieKey(word[idx..].to_vec())) {
                        Some(particle_node) if particle_node.len() > 0 => {
                            let (_, &p_idx) = particle_node.iter().next().expect("no value?");
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
    if let (Some(next_particle_idx),
            Some(padding),
            Some(overlap_word)) = (best_next_particle_idx, best_padding, best_overlap_word) {
        return Next {
            next_idx: next_particle_idx,
            edge: Edge::Padded { padding: padding, overlap_word: overlap_word },
        }
    } else {
        unreachable!()
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

        state.sanity_check();

        let idx = rng.gen_range(0, state.unconnected_on_right.len());
        let particle_idx = state.unconnected_on_right.swap_remove(idx);

        let chain_start_particle_idx = match state.particles[particle_idx].next {
            Ok(_) => unreachable!(),
            Err(ref no_next) => no_next.chain_start_idx,
        };

        // Special case when we are almost done. We need to choose the starticle chain.
        if particles_trie.len() == 1 && chain_start_particle_idx != state.starticle_idx {
            state.unconnected_on_right.push(particle_idx);
            continue;
        }


        let best_next = {
            let particle = &state.particles[particle_idx];
            let chain_start_particle = &state.particles[chain_start_particle_idx];
            let chain_start_particle_trie_key = BytesTrieKey(chain_start_particle.chars.clone());
            // temporarily remove chain_start_particle from particles_trie, to avoid forming a cycle.
            if chain_start_particle_idx != state.starticle_idx {
                particles_trie.remove(&chain_start_particle_trie_key);
            }

            let best_next = find_next(state, words_trie, particle, &particles_trie);

            if chain_start_particle_idx != state.starticle_idx {
                particles_trie.insert(chain_start_particle_trie_key, chain_start_particle_idx);
            }

            best_next
        };

        state.score += best_next.edge.score();
        let next_particle_idx = best_next.next_idx;
        assert!(state.unconnected_on_left.remove(&next_particle_idx));

        {
            let particle = &mut state.particles[particle_idx];
            particle.next = Ok(best_next);
        }

        let chain_end_particle_idx = {
            let next_particle = &mut state.particles[next_particle_idx];
            let chain_end_particle_idx = match next_particle.prev {
                Ok(_) => unreachable!(),
                Err(ref no_prev) => {
                    no_prev.chain_end_idx
                }
            };
            next_particle.prev = Ok(Prev { prev_idx: particle_idx, });
            chain_end_particle_idx
        };

        // Now, update the chain_end_idx at the chain start,

        {
            let chain_start = &mut state.particles[chain_start_particle_idx];
            match chain_start.prev {
                Ok(_) => unreachable!(),
                Err(ref mut no_prev) => {
                    no_prev.chain_end_idx = chain_end_particle_idx;
                }
            }
        }

        // and update the chain_start_idx at the chain end.
        {
            let chain_end = &mut state.particles[chain_end_particle_idx];
            match chain_end.next {
                Ok(_) => unreachable!(),
                Err(ref mut no_next) => {
                    no_next.chain_start_idx = chain_start_particle_idx;
                }
            }
        }

        {
            let next_particle = &state.particles[next_particle_idx];
            particles_trie.remove(&BytesTrieKey(next_particle.chars.clone()));
        }
        if 0 == (state.unconnected_on_right.len() % 100) {
            println!("left: {}. score: {}", state.unconnected_on_right.len(), state.score);
        }
    }
}

fn _accept_new_state<R>(e0: usize, e1: usize, temp: f64, rng: &mut R) -> bool
    where R: rand::Rng
{
    if e1 < e0 {
        true
    } else {
        let p = (-((e1 - e0) as f64) / temp).exp();
        rng.gen_range(0.0, 1.0) < p
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


    let mut urandom = try!(::std::fs::File::open("/dev/urandom"));
    let mut seed: [u32; 4] = [0; 4];
    for idx in 0..4 {
        seed[idx] = try!(urandom.read_u32::<LittleEndian>());
    }
    println!("seed {:?}", seed);

    let mut rng: rand::XorShiftRng = rand::SeedableRng::from_seed(seed);

    coalesce(&mut state, &words_trie, &mut rng);
    try!(write_portmantout(&state));

    loop {
        let mut new_state = state.clone();
        break_chains(&mut new_state, &mut rng);
        coalesce(&mut new_state, &words_trie, &mut rng);

        println!("new state's score: {}", new_state.score);
        if new_state.score < state.score {
            state = new_state;
            try!(write_portmantout(&state));
        }

    }

    Ok(())
}

pub fn main() {
    match main_result() {
        Ok(()) => {}
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
