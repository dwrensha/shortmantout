//! Receives an (ascii, newline-delimited) wordlist on stdin. Removes all words that are contained in
//! other words. Writes the resulting word list to stdout.
//!

pub fn contains_subsequence(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    'outer: for start in 0..(haystack.len() + 1 - needle.len()) {
        for idx in 0..needle.len() {
            if needle[idx] != haystack[start + idx] {
                continue 'outer;
            }
        }
        return true;
    }
    return false;
}

#[test]
fn test_subseq() {
    assert!(contains_subsequence(&[1,3,4], &[3,4]));
    assert!(!contains_subsequence(&[1,3,4], &[3,5]));
    assert!(!contains_subsequence(&[1,3,4], &[3,4,5]));
}
