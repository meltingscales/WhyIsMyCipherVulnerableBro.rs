//! Frequency statistics over letters — the backbone of every classical break.
//! All functions are pure and case-fold letters, ignoring everything else.

/// Standard English letter frequencies for `a..=z`, in percent.
pub const ENGLISH_FREQ: [f64; 26] = [
    8.167, 1.492, 2.782, 4.253, 12.702, 2.228, 2.015, 6.094, 6.966, 0.153, 0.772, 4.025, 2.406,
    6.749, 7.507, 1.929, 0.095, 5.987, 6.327, 9.056, 2.758, 0.978, 2.360, 0.150, 1.974, 0.074,
];

/// Natural English prose used as the canonical plaintext sample everywhere
/// (autocrack victims, traffic plaintext, math-view charts).
pub const ENGLISH_SAMPLE: &str = "Cryptanalysis is the study of finding meaning in \
encrypted messages without access to the secret key. Classical ciphers hide the shape of \
language behind substitutions and rearrangements, but they rarely erase it. English text is \
lumpy: some letters appear constantly while others almost never show up, and this unevenness \
survives encryption whenever each plaintext letter always becomes the same ciphertext letter. \
Frequency analysis turns that survival into an attack. Count the letters, compare the counts \
against what English looks like, and the key often falls out immediately.";

/// Letter counts for `a..=z`, case-folded; non-letters ignored.
pub fn letter_counts(text: &str) -> [usize; 26] {
    let mut counts = [0usize; 26];
    for c in text.chars().filter(|c| c.is_ascii_alphabetic()) {
        counts[(c.to_ascii_lowercase() as u8 - b'a') as usize] += 1;
    }
    counts
}

/// Chi-squared distance between the text's letter distribution and English.
/// Smaller means more English-like; `f64::INFINITY` when the text has no letters.
pub fn chi_squared(text: &str) -> f64 {
    let counts = letter_counts(text);
    let total: usize = counts.iter().sum();
    if total == 0 {
        return f64::INFINITY;
    }
    let total = total as f64;
    ENGLISH_FREQ
        .iter()
        .zip(&counts)
        .map(|(&expected_percent, &count)| {
            let expected = expected_percent / 100.0;
            let observed = count as f64 / total;
            (observed - expected).powi(2) / expected
        })
        .sum()
}

/// Index of coincidence: the probability that two randomly drawn letters from
/// the text match. English sits near 0.066; flat random text near 0.038.
pub fn index_of_coincidence(text: &str) -> f64 {
    let counts = letter_counts(text);
    let total: usize = counts.iter().sum();
    if total < 2 {
        return 0.0;
    }
    // u128 with saturating subtraction: counts squared can exceed usize, and
    // absent letters (count 0) must contribute 0 pairs, not wrap.
    let pairs: u128 = counts
        .iter()
        .map(|&c| {
            let c = c as u128;
            c * c.saturating_sub(1)
        })
        .sum();
    let total = total as u128;
    pairs as f64 / (total * (total - 1)) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Local rotation helper so freq tests do not depend on the caesar module.
    fn rotate(text: &str, shift: u8) -> String {
        text.chars()
            .map(|c| match c {
                'a'..='z' => ((c as u8 - b'a' + shift) % 26 + b'a') as char,
                'A'..='Z' => ((c as u8 - b'A' + shift) % 26 + b'A') as char,
                other => other,
            })
            .collect()
    }

    #[test]
    fn english_sample_is_its_own_best_chi_squared_fit() {
        let baseline = chi_squared(ENGLISH_SAMPLE);
        for shift in 1u8..26 {
            let rotated = rotate(ENGLISH_SAMPLE, shift);
            assert!(
                chi_squared(&rotated) > baseline,
                "rotation by {shift} scored at least as English as the plaintext"
            );
        }
    }

    #[test]
    fn index_of_coincidence_is_in_english_band() {
        let ioc = index_of_coincidence(ENGLISH_SAMPLE);
        assert!(
            (0.055..=0.075).contains(&ioc),
            "IoC {ioc} outside English band"
        );
    }

    #[test]
    fn flat_26_letter_text_scores_lower_ioc_than_english() {
        // Uniform over all 26 letters: IoC approaches 1/26 (~0.038), below
        // English's lumpy ~0.066. (A uniform text over only 10 distinct letters
        // would score ~0.1 — *above* English — since draws collide more often.)
        let flat: String = "abcdefghijklmnopqrstuvwxyz".repeat(30);
        assert!(index_of_coincidence(&flat) < index_of_coincidence(ENGLISH_SAMPLE));
    }

    #[test]
    fn single_letter_text_has_perfect_ioc_and_empty_has_zero() {
        assert!((index_of_coincidence(&"q".repeat(50)) - 1.0).abs() < 1e-12);
        assert_eq!(index_of_coincidence("123 !?"), 0.0);
        assert_eq!(chi_squared("123 !?"), f64::INFINITY);
    }
}
