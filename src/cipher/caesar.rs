//! Caesar cipher: shift letters by a fixed amount, and the brute-force break
//! that scores all 26 shifts against English letter frequencies.

use super::freq::chi_squared;

/// Shift `text` forward by `shift` (0-25), preserving case; non-letters pass through.
pub fn encrypt(text: &str, shift: u8) -> String {
    shift_letters(text, shift)
}

/// Shift `text` backward by `shift`, the exact inverse of [`encrypt`].
pub fn decrypt(text: &str, shift: u8) -> String {
    shift_letters(text, 26 - shift % 26)
}

fn shift_letters(text: &str, shift: u8) -> String {
    let shift = shift % 26;
    text.chars()
        .map(|c| match c {
            'a'..='z' => (((c as u8 - b'a') + shift) % 26 + b'a') as char,
            'A'..='Z' => (((c as u8 - b'A') + shift) % 26 + b'A') as char,
            other => other,
        })
        .collect()
}

/// One tried candidate: the shift, its chi-squared score, and what it decrypts to.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub shift: u8,
    pub score: f64,
    pub plaintext: String,
}

/// All 26 shifts scored by chi-squared distance from English, best (lowest) first.
pub fn crack(ciphertext: &str) -> Vec<Candidate> {
    let mut candidates: Vec<Candidate> = (0..26)
        .map(|shift| {
            let plaintext = decrypt(ciphertext, shift);
            Candidate {
                shift,
                score: chi_squared(&plaintext),
                plaintext,
            }
        })
        .collect();
    candidates.sort_by(|a, b| a.score.partial_cmp(&b.score).expect("scores are finite"));
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_for_every_shift() {
        for shift in 0..26 {
            let ct = encrypt("Attack at Dawn, Zz!", shift);
            assert_eq!(decrypt(&ct, shift), "Attack at Dawn, Zz!", "shift {shift}");
        }
    }

    #[test]
    fn known_shift_produces_known_ciphertext() {
        assert_eq!(encrypt("abc", 2), "cde");
        assert_eq!(encrypt("xyz", 3), "abc"); // wraps
        assert_eq!(decrypt("cde", 2), "abc");
    }

    #[test]
    fn crack_recovers_the_true_key_and_plaintext() {
        const SAMPLE: &str = "The quick brown fox jumps over the lazy dog and then \
            continues on its way through the meadow because samples need length \
            for statistics to mean anything at all";
        let ciphertext = encrypt(SAMPLE, 11);
        let best = &crack(&ciphertext)[0];
        // Recovering "shift 11" means finding key = 11; plaintext matches exactly.
        assert_eq!(best.shift, 11);
        // Whitespace differs only where the source had hard line-wrap spaces.
        assert_eq!(
            best.plaintext.replace("  ", " "),
            SAMPLE.replace("            ", "")
        );
    }

    #[test]
    fn crack_orders_best_first() {
        let candidates = crack("a");
        assert!(candidates.windows(2).all(|w| w[0].score <= w[1].score));
    }
}
