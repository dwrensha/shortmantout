extern crate carrycoat;


pub fn main() {
    use std::io::BufRead;

    let args : Vec<String> = ::std::env::args().collect();
    if args.len() != 4 {
        println!("usage: {} PORTMANTEAU_FILE WORDLIST_FILE NORMALIZED_WORDLIST_FILE", args[0]);
        return;
    }


    let mut portmanteau_reader = ::std::fs::File::open(&args[1]).unwrap();
}
