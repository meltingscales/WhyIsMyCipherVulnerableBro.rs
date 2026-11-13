//! Logjam, scaled down: toy Diffie–Hellman in a small safe-prime group and
//! the baby-step giant-step discrete log that kills it. Real Logjam solved
//! dlog in 512-bit export groups after precomputing tables for the handful
//! of primes everyone shared. The demo group is ~2^40, so BSGS runs live in
//! seconds while telling the same story: recover the exponent, recompute
//! the shared secret, decrypt the session.

use std::collections::HashMap;

use super::{modinv, modpow, mulmod};

/// The demo "export group": p = 2q + 1 with q prime, g generating the full
/// multiplicative group.
pub const P: u64 = 1_099_511_628_443;
pub const G: u64 = 5;
/// The client's private exponent (public value published in the exchange).
pub const CLIENT_EXP: u64 = 8_675_309;
/// The server's private exponent — the discrete-log target.
pub const SERVER_EXP: u64 = 314_159_265;
/// The session bytes the shared secret protects.
pub const SESSION: &[u8] = b"GET /export HTTP/1.1";

/// A second, smaller group for the cost chart's second BSGS data point:
/// a 38-bit safe prime and a fixed target exponent.
pub const P2: u64 = 137_438_954_447;
pub const G2: u64 = 5;
pub const Y2: u64 = 271_828;

/// The second group's public target B₂ = g₂^y₂.
pub fn second_target() -> u64 {
    modpow(G2, Y2, P2)
}

/// The client's public value A = g^a.
pub fn client_public() -> u64 {
    modpow(G, CLIENT_EXP, P)
}

/// The server's public value B = g^y — what the attacker sees.
pub fn server_public() -> u64 {
    modpow(G, SERVER_EXP, P)
}

/// The shared secret s = A^y = B^a, derivable once y is known.
pub fn shared_secret() -> u64 {
    modpow(client_public(), SERVER_EXP, P)
}

/// A keystream block from the secret: its 8 LE bytes.
pub fn keystream(secret: u64) -> [u8; 8] {
    secret.to_le_bytes()
}

/// The captured session ciphertext: plaintext XOR keystream(secret).
pub fn session_ciphertext() -> Vec<u8> {
    let key = keystream(shared_secret());
    SESSION
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

/// Decrypt the session under a recovered secret.
pub fn decrypt_session(secret: u64) -> Vec<u8> {
    let key = keystream(secret);
    session_ciphertext()
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

/// Expected BSGS step count: ≈ √(order), order = p - 1 for this generator.
pub fn expected_steps() -> f64 {
    ((P - 1) as f64).sqrt()
}

/// Baby-step giant-step over a prime-order-ish group, with the step counter
/// the demo streams. Deterministic for fixed (p, g, target).
#[derive(Clone)]
pub struct BsgsSolver {
    p: u64,
    g: u64,
    target: u64,
    /// m = ⌈√(p-1)⌉: baby steps cover g^0..g^(m-1).
    pub m: u64,
    /// Baby table g^j → j, built incrementally.
    babies: HashMap<u64, u64>,
    baby_value: u64,
    /// How many baby steps have been inserted (progress readout).
    pub baby_index: u64,
    /// g^(-m), the giant stride multiplier.
    giant_stride: u64,
    /// Current giant probe value: target · (g^(-m))^i.
    giant_value: u64,
    giant_index: u64,
    /// Total steps: baby inserts + giant probes.
    pub steps: u64,
    /// Set when the discrete log is found.
    pub exponent: Option<u64>,
}

impl PartialEq for BsgsSolver {
    fn eq(&self, other: &Self) -> bool {
        self.p == other.p
            && self.g == other.g
            && self.target == other.target
            && self.m == other.m
            && self.baby_index == other.baby_index
            && self.giant_index == other.giant_index
            && self.steps == other.steps
            && self.exponent == other.exponent
    }
}

impl std::fmt::Debug for BsgsSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BsgsSolver")
            .field("p", &self.p)
            .field("g", &self.g)
            .field("target", &self.target)
            .field("m", &self.m)
            .field("steps", &self.steps)
            .field("exponent", &self.exponent)
            .finish_non_exhaustive()
    }
}

impl BsgsSolver {
    pub fn new(p: u64, g: u64, target: u64) -> Self {
        let m = ((p - 1) as f64).sqrt().ceil() as u64;
        Self {
            p,
            g,
            target,
            m,
            babies: HashMap::new(),
            baby_value: 1,
            baby_index: 0,
            giant_stride: modpow(modinv(g, p), m, p),
            giant_value: target,
            giant_index: 0,
            steps: 0,
            exponent: None,
        }
    }

    /// The demo solver against the server's public value.
    pub fn demo() -> Self {
        Self::new(P, G, server_public())
    }

    /// Advance up to `n` steps (baby inserts first, then giant probes),
    /// stopping at the solution. Returns steps actually performed.
    pub fn step(&mut self, n: u64) -> u64 {
        let mut done = 0;
        while done < n && self.exponent.is_none() {
            if self.baby_index < self.m {
                self.babies.insert(self.baby_value, self.baby_index);
                self.baby_index += 1;
                self.baby_value = mulmod(self.baby_value, self.g, self.p);
            } else {
                // Probe the current giant value, then stride.
                if let Some(&j) = self.babies.get(&self.giant_value) {
                    self.exponent = Some(self.giant_index * self.m + j);
                    return done;
                }
                self.giant_value = mulmod(self.giant_value, self.giant_stride, self.p);
                self.giant_index += 1;
            }
            self.steps += 1;
            done += 1;
        }
        // One final probe if budget ran out exactly at the boundary.
        if self.exponent.is_none()
            && self.baby_index >= self.m
            && let Some(&j) = self.babies.get(&self.giant_value)
        {
            self.exponent = Some(self.giant_index * self.m + j);
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_values_match_offline_computation() {
        // Verified independently: A, B, and the shared secret.
        assert_eq!(client_public(), 554_457_436_274);
        assert_eq!(server_public(), 972_255_028_306);
        assert_eq!(shared_secret(), 687_544_775_438);
    }

    #[test]
    fn session_round_trips_under_the_secret() {
        assert_eq!(decrypt_session(shared_secret()), SESSION.to_vec());
        assert_ne!(session_ciphertext(), SESSION.to_vec(), "it is encrypted");
    }

    #[test]
    fn bsgs_recovers_the_server_exponent_in_the_expected_band() {
        let mut solver = BsgsSolver::demo();
        while solver.exponent.is_none() && solver.steps < 5_000_000 {
            solver.step(65_536);
        }
        let found = solver.exponent.expect("BSGS terminates on the demo group");
        assert_eq!(found, SERVER_EXP, "finds the true exponent (y < q)");
        assert_eq!(modpow(G, found, P), server_public(), "and it reproduces B");
        let expected = expected_steps();
        assert!(
            (expected / 2.0..=2.0 * expected).contains(&(solver.steps as f64)),
            "{} steps outside [{expected:.0}, {:.0}]",
            solver.steps,
            2.0 * expected
        );
    }

    #[test]
    fn bsgs_is_deterministic() {
        let run = || {
            let mut s = BsgsSolver::demo();
            while s.exponent.is_none() {
                s.step(65_536);
            }
            (s.exponent, s.steps)
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn recovered_secret_unlocks_the_session() {
        let mut solver = BsgsSolver::demo();
        while solver.exponent.is_none() {
            solver.step(131_072);
        }
        let y = solver.exponent.unwrap();
        // The attacker's path: y → shared secret = A^y → plaintext.
        let attacker_secret = modpow(client_public(), y, P);
        assert_eq!(attacker_secret, shared_secret());
        assert_eq!(decrypt_session(attacker_secret), SESSION.to_vec());
    }
}
