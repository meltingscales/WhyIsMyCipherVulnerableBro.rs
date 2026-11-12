//! Pure classical-cipher implementations and their cryptanalysis.
//! No TUI types here: modes depend on `cipher`, never the reverse.

pub mod caesar;
pub mod dlog;
pub mod freq;
pub mod rsa;
pub mod sweet32;
pub mod vigenere;
pub mod xor;

/// `(a · b) mod m`, safe for any `u64` operands via 128-bit intermediates.
pub fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// `(base^exp) mod m` by square-and-multiply.
pub fn modpow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    assert!(m > 0, "modulus must be positive");
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mulmod(result, base, m);
        }
        base = mulmod(base, base, m);
        exp >>= 1;
    }
    result
}

/// Modular inverse of `a` mod `m` by extended Euclid; panics if not coprime.
pub fn modinv(a: u64, m: u64) -> u64 {
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
    }
    assert_eq!(old_r, 1, "a and m must be coprime");
    let inv = old_s.rem_euclid(m as i128);
    inv as u64
}

/// Euclidean gcd.
pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mulmod_handles_full_range() {
        // Products near u128 overflow boundaries stay exact.
        let big = u64::MAX;
        assert_eq!(mulmod(big, big, u64::MAX), 0);
        assert_eq!(mulmod(6, 7, 10), 2);
        assert_eq!(mulmod(0, big, 12345), 0);
    }

    #[test]
    fn modpow_matches_known_values() {
        assert_eq!(modpow(2, 10, 1000), 1024 % 1000);
        assert_eq!(modpow(5, 0, 13), 1);
        assert_eq!(modpow(0, 5, 13), 0);
        // Verified offline: 5^314159265 mod 1099511628443.
        assert_eq!(modpow(5, 314_159_265, 1_099_511_628_443), 972_255_028_306);
    }

    #[test]
    fn modinv_round_trips() {
        for (a, m) in [(3u64, 11u64), (65_537, 2_147_483_628), (7, 40)] {
            let inv = modinv(a, m);
            assert_eq!(mulmod(a, inv, m), 1);
        }
    }

    #[test]
    fn gcd_basics() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(
            gcd(2_147_483_628, 2_147_483_658),
            gcd(2_147_483_658, 2_147_483_628)
        );
        assert_eq!(gcd(7, 13), 1);
    }
}
