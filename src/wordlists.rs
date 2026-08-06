include!(concat!(env!("OUT_DIR"), "/wordlists.rs"));

pub fn by_letter_count(letter_count: usize) -> Option<&'static [&'static str]> {
    WORDLISTS
        .iter()
        .find_map(|(count, wordlist)| (*count == letter_count).then_some(*wordlist))
}
