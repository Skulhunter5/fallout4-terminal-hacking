include!(concat!(env!("OUT_DIR"), "/wordlists.rs"));

pub fn by_word_length(length: usize) -> Option<&'static [&'static str]> {
    WORDLISTS
        .iter()
        .find_map(|(word_length, wordlist)| (*word_length == length).then_some(*wordlist))
}
