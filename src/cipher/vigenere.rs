//! Vigenère cipher: a running Caesar shift keyed by a repeating word. The
//! break has two stages — find the key length with the index of coincidence,
//! then solve each key column as an independent Caesar via chi-squared.

use super::freq::{chi_squared, index_of_coincidence};

/// Encrypt with a repeating alphabetic key; letters only are shifted (their
/// positions count toward key advancement), case is preserved.
pub fn encrypt(text: &str, key: &str) -> String {
    apply_shifts(text, &key_shifts(key))
}

/// Exact inverse of [`encrypt`]: shifts each letter backward by its key letter.
pub fn decrypt(text: &str, key: &str) -> String {
    let negated: Vec<u8> = key_shifts(key).iter().map(|&s| (26 - s) % 26).collect();
    apply_shifts(text, &negated)
}

fn key_shifts(key: &str) -> Vec<u8> {
    let shifts: Vec<u8> = key
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase() as u8 - b'a')
        .collect();
    assert!(!shifts.is_empty(), "key must contain at least one letter");
    shifts
}

/// Apply per-letter shifts to letters only; the key position advances on
/// letters alone so punctuation and spaces never consume key material.
fn apply_shifts(text: &str, shifts: &[u8]) -> String {
    text.chars()
        .fold(
            (String::with_capacity(text.len()), 0usize),
            |(mut out, i), c| {
                let shifted = match c {
                    'a'..='z' | 'A'..='Z' => {
                        let s = shifts[i % shifts.len()];
                        let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                        (((c as u8 - base) + s) % 26 + base) as char
                    }
                    other => other,
                };
                out.push(shifted);
                (out, if c.is_ascii_alphabetic() { i + 1 } else { i })
            },
        )
        .0
}

/// One scored guess at the key length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyLengthScore {
    pub length: usize,
    /// Average index of coincidence across the length's slices; near English's
    /// ~0.066 when the slice alignment matches the true key period.
    pub ioc: f64,
}

/// Score key lengths `min..=max` by average slice IoC, best (highest) first.
pub fn key_length_scan(ciphertext: &str, min: usize, max: usize) -> Vec<KeyLengthScore> {
    let letters: Vec<u8> = ciphertext
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase() as u8)
        .collect();
    let mut scores: Vec<KeyLengthScore> = (min..=max)
        .map(|length| {
            let ioc = (0..length)
                .map(|offset| {
                    let slice: String = letters[offset..]
                        .iter()
                        .step_by(length)
                        .map(|&b| b as char)
                        .collect();
                    index_of_coincidence(&slice)
                })
                .sum::<f64>()
                / length as f64;
            KeyLengthScore { length, ioc }
        })
        .collect();
    scores.sort_by(|a, b| b.ioc.partial_cmp(&a.ioc).expect("IoC values are finite"));
    scores
}

/// One tried key letter for one key column, scored by chi-squared of the
/// column decrypted under it.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnCandidate {
    pub key_letter: char,
    pub score: f64,
}

/// All 26 key letters for column `column` of a `key_len`-period Vigenère,
/// best (lowest chi-squared) first.
pub fn crack_column(ciphertext: &str, key_len: usize, column: usize) -> Vec<ColumnCandidate> {
    let col_letters: String = ciphertext
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .enumerate()
        .filter(|(i, _)| i % key_len == column)
        .map(|(_, c)| c.to_ascii_lowercase())
        .collect();
    let mut candidates: Vec<ColumnCandidate> = (0..26u8)
        .map(|shift| {
            let decrypted: String = col_letters
                .chars()
                .map(|c| (((c as u8 - b'a') + (26 - shift) % 26) % 26 + b'a') as char)
                .collect();
            ColumnCandidate {
                key_letter: (b'a' + shift) as char,
                score: chi_squared(&decrypted),
            }
        })
        .collect();
    candidates.sort_by(|a, b| a.score.partial_cmp(&b.score).expect("scores are finite"));
    candidates
}

/// Full break at a known key length: best letter per column, plus the plaintext.
pub fn crack(ciphertext: &str, key_len: usize) -> (String, String) {
    let key: String = (0..key_len)
        .map(|col| crack_column(ciphertext, key_len, col)[0].key_letter)
        .collect();
    let plaintext = decrypt(ciphertext, &key);
    (key, plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cipher::freq::ENGLISH_SAMPLE;

    const KEY: &str = "lemon";

    #[test]
    fn round_trips() {
        let ct = encrypt(ENGLISH_SAMPLE, KEY);
        assert_eq!(decrypt(&ct, KEY), ENGLISH_SAMPLE);
    }

    #[test]
    fn known_vector() {
        // Classic textbook vector: ATTACKATDAWN + LEMON → LXFOPVEFRNHR
        assert_eq!(encrypt("attackatdawn", "lemon"), "lxfopvefrnhr");
        assert_eq!(decrypt("lxfopvefrnhr", "lemon"), "attackatdawn");
    }

    #[test]
    fn non_letters_do_not_advance_the_key() {
        assert_eq!(encrypt("a b", "b"), "b c");
    }

    #[test]
    fn scan_finds_the_true_key_length() {
        let ct = encrypt(ENGLISH_SAMPLE, KEY);
        let scan = key_length_scan(&ct, 1, 12);
        // Multiples of the true period spike too (slices stay aligned), so the
        // classic heuristic is the SMALLEST length entering the English band.
        let smallest_spiked = scan
            .iter()
            .filter(|s| s.ioc >= 0.06)
            .map(|s| s.length)
            .min()
            .expect("some length spikes");
        assert_eq!(smallest_spiked, 5);
    }

    #[test]
    fn crack_recovers_the_key_and_plaintext() {
        let ct = encrypt(ENGLISH_SAMPLE, KEY);
        let (found_key, found_pt) = crack(&ct, 5);
        assert_eq!(found_key, KEY);
        assert_eq!(found_pt, ENGLISH_SAMPLE);
    }
}
