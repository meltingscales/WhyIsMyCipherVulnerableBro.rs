//! FREAK, scaled down: toy export-grade RSA and the factoring attack that
//! kills it. Real FREAK factored 512-bit RSA_EXPORT moduli — infeasible in
//! an afternoon, tractable for the patient. The demo's modulus is 62 bits,
//! so Pollard's rho factors it live in seconds while telling the same
//! story: factor N once, derive the private key, decrypt the secret.

use super::{gcd, modinv, modpow};

/// Fixed demo primes (both ~2^31), deterministic every run.
pub const P: u64 = 2_147_483_629;
pub const Q: u64 = 2_147_483_659;
/// The "export-grade" modulus the attack targets.
pub const N: u64 = P * Q;
/// Standard public exponent.
pub const E: u64 = 65_537;
/// The premaster secret FREAK steals, toy-sized to fit one RSA block.
pub const SECRET: &[u8] = b"PMS-42!";

/// A second fixed semiprime (one bit larger), for the cost chart's second
/// rho data point. Deterministic every run.
pub const P2: u64 = 3_037_000_013;
pub const Q2: u64 = 3_037_000_039;
pub const N2: u64 = P2 * Q2;

/// λ(N) = lcm(p-1, q-1).
pub fn lambda() -> u64 {
    (P - 1) / gcd(P - 1, Q - 1) * (Q - 1)
}

/// The private exponent d = e⁻¹ mod λ(N) — what factoring N unlocks.
pub fn private_exponent() -> u64 {
    modinv(E, lambda())
}

/// The secret bytes as one little-endian integer (< N by construction).
pub fn secret_int() -> u64 {
    let mut buf = [0u8; 8];
    buf[..SECRET.len()].copy_from_slice(SECRET);
    u64::from_le_bytes(buf)
}

/// Textbook RSA: c = m^e mod N.
pub fn encrypt_int(m: u64) -> u64 {
    modpow(m, E, N)
}

/// Textbook RSA: m = c^d mod N.
pub fn decrypt_int(c: u64, d: u64) -> u64 {
    modpow(c, d, N)
}

/// The demo's captured export ciphertext: the premaster secret under e.
pub fn captured_ciphertext() -> u64 {
    encrypt_int(secret_int())
}

/// The expected number of rho iterations, ≈ N^(1/4) for balanced semiprimes.
pub fn expected_iterations() -> f64 {
    (N as f64).powf(0.25)
}

/// Pollard's rho (Floyd cycle detection, f(x) = x² + 1, seeded at 2), with
/// the iteration counter the demo streams. Deterministic: fixed walk.
///
/// `PartialEq`/`Debug` expose the observable attack state.
pub struct RhoSolver {
    modulus: u64,
    tortoise: u64,
    hare: u64,
    /// Total f-steps taken so far (one tortoise step per counted iteration).
    pub iterations: u64,
    /// Set when a nontrivial factor is found.
    pub factor: Option<u64>,
}

impl PartialEq for RhoSolver {
    fn eq(&self, other: &Self) -> bool {
        self.modulus == other.modulus
            && self.tortoise == other.tortoise
            && self.hare == other.hare
            && self.iterations == other.iterations
            && self.factor == other.factor
    }
}

impl std::fmt::Debug for RhoSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RhoSolver")
            .field("modulus", &self.modulus)
            .field("iterations", &self.iterations)
            .field("factor", &self.factor)
            .finish_non_exhaustive()
    }
}

impl RhoSolver {
    pub fn new(modulus: u64) -> Self {
        Self {
            modulus,
            tortoise: 2,
            hare: 2,
            iterations: 0,
            factor: None,
        }
    }

    /// The demo solver over [`N`].
    pub fn demo() -> Self {
        Self::new(N)
    }

    /// Advance up to `n` iterations, stopping at the factor. Returns the
    /// number of iterations actually performed.
    pub fn step(&mut self, n: u64) -> u64 {
        let mut done = 0;
        while done < n && self.factor.is_none() {
            self.tortoise = self.f(self.tortoise);
            self.hare = self.f(self.f(self.hare));
            self.iterations += 1;
            done += 1;
            let d = gcd(self.tortoise.abs_diff(self.hare), self.modulus);
            if d > 1 && d < self.modulus {
                self.factor = Some(d);
            } else if d == self.modulus {
                // Degenerate cycle for this seed; the demo constants avoid
                // it, but a general caller deserves a defined outcome.
                self.factor = Some(1);
            }
        }
        done
    }

    fn f(&self, x: u64) -> u64 {
        (super::mulmod(x, x, self.modulus) + 1) % self.modulus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_constants_are_consistent() {
        assert_eq!(N, P * Q);
        assert!(secret_int() < N, "secret must fit one block");
    }

    #[test]
    fn encryption_round_trips_with_the_private_exponent() {
        let d = private_exponent();
        assert_eq!(super::super::mulmod(E, d, lambda()), 1);
        for m in [secret_int(), 2, 42, N - 1] {
            assert_eq!(decrypt_int(encrypt_int(m), d), m, "m = {m}");
        }
    }

    #[test]
    fn rho_factors_the_demo_modulus_in_the_expected_band() {
        let mut solver = RhoSolver::demo();
        while solver.factor.is_none() && solver.iterations < 2_000_000 {
            solver.step(4096);
        }
        let factor = solver.factor.expect("rho terminates quickly on 62-bit N");
        assert!(factor == P || factor == Q, "found a true prime factor");
        let expected = expected_iterations();
        assert!(
            (expected / 2.0..=4.0 * expected).contains(&(solver.iterations as f64)),
            "{} iterations outside [{:.0}, {:.0}]",
            solver.iterations,
            expected / 2.0,
            4.0 * expected
        );
    }

    #[test]
    fn rho_is_deterministic() {
        let run = || {
            let mut s = RhoSolver::demo();
            while s.factor.is_none() {
                s.step(4096);
            }
            (s.factor, s.iterations)
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn full_freak_chain_recovers_the_secret() {
        // factor → primes → d → decrypt, the whole kill chain.
        let mut solver = RhoSolver::demo();
        while solver.factor.is_none() {
            solver.step(8192);
        }
        let p = solver.factor.unwrap();
        let q = N / p;
        let d = modinv(E, (p - 1) / gcd(p - 1, q - 1) * (q - 1));
        let m = decrypt_int(captured_ciphertext(), d);
        let mut buf = m.to_le_bytes();
        buf[..SECRET.len()].copy_from_slice(&m.to_le_bytes()[..SECRET.len()]);
        assert_eq!(&buf[..SECRET.len()], SECRET, "premaster recovered");
    }
}
