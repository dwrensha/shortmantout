use std::collections::hash_map::HashMap;

fn main_result() -> ::std::result::Result<(), Box<::std::error::Error>> {
    use std::io::{BufRead, Read};

    let args : Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} PARTICLES_FILE JOINERS_FILE", args[0]);
        return Ok(());
    }

    let mut particles = Vec::new();
    for maybe_word in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[1]))).split('\n' as u8) {
        particles.push(try!(maybe_word));
    }

    let mut joiners = HashMap::<(u8, u8), Vec<u8>>::new();
    for maybe_joiner in ::std::io::BufReader::new(try!(::std::fs::File::open(&args[2]))).split('\n' as u8) {
        let joiner = try!(maybe_joiner);
        let key = (*joiner.first().unwrap(), *joiner.last().unwrap());
        if !joiners.contains_key(&key) {
            joiners.insert(key, joiner);
        }
    }

    let mut portmantout = Vec::new();

    let mut last = None;
    for particle in &particles {
        match last {
            Some(&a) => {
                let &b = particle.first().expect("empty particle?");
                let ref joiner = joiners[&(a,b)];
                for idx in 1..(joiner.len()-1){
                    portmantout.push(joiner[idx]);
                }

            }
            None => {}
        }

        last = Some(particle.last().expect("empty particle?"));
        for &c in particle {
            portmantout.push(c);
        }
    }

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
