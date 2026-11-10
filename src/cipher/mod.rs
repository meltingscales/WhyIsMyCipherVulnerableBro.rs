//! Pure classical-cipher implementations and their cryptanalysis.
//! No TUI types here: modes depend on `cipher`, never the reverse.

pub mod caesar;
pub mod freq;
pub mod vigenere;
pub mod xor;
