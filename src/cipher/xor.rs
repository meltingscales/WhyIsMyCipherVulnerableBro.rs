//! Repeating-key XOR — cryptopals set 1 territory. Encryption and decryption
//! are the same operation; the break guesses the key length by normalized
//! Hamming distance, then solves each key byte as an independent single-byte
//! XOR scored as printable English.

use super::freq::chi_squared;

/// XOR `data` with `key` repeating. This is both encrypt and decrypt.
pub fn apply(data: &[u8], key: &[u8]) -> Vec<u8> {
    assert!(!key.is_empty(), "key must be non-empty");
    data.iter()
        .zip(key.iter().cycle())
        .map(|(d, k)| d ^ k)
        .collect()
}

/// Score decoded bytes as English: chi-squared over the letters, a space-
/// frequency term (English is ~17% spaces; a wrong key byte scatters them),
/// and a heavy penalty per non-printable byte. Lower is better.
fn english_score(bytes: &[u8]) -> f64 {
    let text: String = bytes
        .iter()
        .map(|&b| {
            if (32..=126).contains(&b) || matches!(b, b'\n' | b'\r' | b'\t') {
                b as char
            } else {
                '\u{fffd}' // marks the byte as unprintable
            }
        })
        .collect();
    let unprintable = text.chars().filter(|&c| c == '\u{fffd}').count();
    let total = text.len().saturating_sub(unprintable);
    let space_term = if total > 0 {
        let observed_space = text.matches(' ').count() as f64 / total as f64;
        (observed_space - 0.17).powi(2) / 0.17
    } else {
        0.0
    };
    unprintable as f64 * 1000.0 + chi_squared(&text) + space_term
}

/// One scored guess at the key length, by how English the best full break at
/// that length reads. Lower score is better.
///
/// The textbook alternative — normalized Hamming distance between blocks —
/// needs captures far longer than this app's simulated sessions to separate
/// true periods from noise; cracking each length and scoring the decryption is
/// decisive at any size the traffic simulator produces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyLengthGuess {
    pub length: usize,
    pub score: f64,
}

/// Rank key lengths `min..=max`: crack at each length and score the decryption
/// as English; best (lowest) first.
pub fn guess_key_length(ciphertext: &[u8], min: usize, max: usize) -> Vec<KeyLengthGuess> {
    let mut guesses: Vec<KeyLengthGuess> = (min..=max)
        .filter(|&len| ciphertext.len() >= len * 4)
        .map(|length| {
            let key = crack_key(ciphertext, length);
            let decrypted = apply(ciphertext, &key);
            KeyLengthGuess {
                length,
                score: english_score(&decrypted),
            }
        })
        .collect();
    guesses.sort_by(|a, b| a.score.partial_cmp(&b.score).expect("scores are finite"));
    guesses
}

/// All 256 key bytes for column `column` of a `key_len`-period XOR, scored as
/// printable English over that column, best (lowest) first.
pub fn crack_column(ciphertext: &[u8], key_len: usize, column: usize) -> Vec<(u8, f64)> {
    let col: Vec<u8> = ciphertext[column..]
        .iter()
        .step_by(key_len)
        .copied()
        .collect();
    let mut candidates: Vec<(u8, f64)> = (0..=255u8)
        .map(|kb| {
            let decoded: Vec<u8> = col.iter().map(|&b| b ^ kb).collect();
            (kb, english_score(&decoded))
        })
        .collect();
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("scores are finite"));
    candidates
}

/// Full break at a known key length: best byte per column.
pub fn crack_key(ciphertext: &[u8], key_len: usize) -> Vec<u8> {
    (0..key_len)
        .map(|col| crack_column(ciphertext, key_len, col)[0].0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A realistic multi-request capture: enough English for the per-column
    /// scoring to see letter and space frequencies clearly.
    const PLAINTEXT: &[u8] = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: \
        curl/8.0\r\nAccept: */*\r\nGET /style.css HTTP/1.1\r\nHost: example.com\r\nReferer: \
        https://example.com/\r\nPOST /login HTTP/1.1\r\nHost: example.com\r\nContent-Type: \
        application/x-www-form-urlencoded\r\nusername=alice&password=hunter2&remember=on\r\n\
        the quick brown fox jumps over the lazy dog and keeps going so there is enough text \
        for statistics to mean anything, because frequency analysis needs volume";
    const KEY: &[u8] = b"abc";

    #[test]
    fn apply_is_its_own_inverse() {
        let ct = apply(PLAINTEXT, KEY);
        assert_eq!(apply(&ct, KEY), PLAINTEXT);
    }

    #[test]
    fn known_xor_vector() {
        assert_eq!(apply(b"abc", &[0x00]), b"abc");
        assert_eq!(apply(b"\x00\x00\x00", b"abc"), b"abc");
        // 0x41 ^ 0x00 = 'A'; 'a' ^ 0x20 = 'A'
        assert_eq!(apply(b"a", &[0x20]), b"A");
    }

    #[test]
    fn crack_key_recovers_the_three_byte_key() {
        let ct = apply(PLAINTEXT, KEY);
        assert_eq!(crack_key(&ct, 3), KEY.to_vec());
    }

    #[test]
    fn guess_key_length_finds_three() {
        let ct = apply(PLAINTEXT, KEY);
        assert_eq!(guess_key_length(&ct, 2, 5)[0].length, 3);
    }

    #[test]
    fn wrong_key_bytes_score_worse() {
        let ct = apply(PLAINTEXT, KEY);
        let best = crack_column(&ct, 3, 0);
        assert_eq!(best[0].0, KEY[0]);
        assert!(best[0].1 < best[255].1);
    }
}
