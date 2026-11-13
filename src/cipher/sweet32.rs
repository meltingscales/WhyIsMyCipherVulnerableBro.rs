//! SWEET32, scaled down until you can watch it: a birthday-collision
//! simulator over block-cipher traffic.
//!
//! Each simulated TLS record is one 16-byte block — a known header, a
//! counter, and a 4-byte secret cookie — encrypted with AES-128 and
//! truncated to the low `DEMO_BLOCK_BITS` bits. Truncation is the whole
//! lesson: the attack never touches AES's strength, only the size of the
//! block space. Real SWEET32 used a 64-bit space (3DES) and needed ~2^32
//! blocks; the demo uses 2^20 so the same curve plays out in seconds.
//!
//! Every third record belongs to the attacker's own session, with a known
//! (all-zero) cookie. When a victim record collides with an own record, the
//! XOR of the two plaintexts cancels the header and counter and hands over
//! the victim's cookie — the known-plaintext leak that is SWEET32's actual
//! impact.

use std::collections::HashMap;

use aes::cipher::{BlockCipherEncrypt, KeyInit};
use aes::{Aes128, Block};

/// Demo block space: 2^20 block values.
pub const DEMO_BLOCK_BITS: u32 = 20;
/// How many records one autocrack tick sends. Sized so the demo's first
/// exploitable collision (~4.7k blocks with the fixed construction) plays
/// out in roughly six seconds.
pub const TICK_STRIDE: usize = 192;
/// Every OWN_INTERVAL-th record is the attacker's own (known-cookie) session.
const OWN_INTERVAL: usize = 3;
/// Fixed demo AES key — the attack is independent of the key's value.
const DEMO_KEY: [u8; 16] = *b"why-is-my-cipher";
/// Known record header, identical in every record so it cancels in XOR.
const HEADER: [u8; 8] = *b"GET / HT";

/// Expected number of blocks until the first collision: √(π/2 · 2^bits).
pub fn first_collision_expectation(bits: u32) -> f64 {
    (std::f64::consts::PI / 2.0 * (1u64 << bits) as f64).sqrt()
}

/// Expected number of colliding pairs after `sent` blocks in a space of
/// `2^bits`: k(k-1)/2 / 2^bits ≈ k²/2^(bits+1).
pub fn expected_collisions(sent: usize, bits: u32) -> f64 {
    let k = sent as f64;
    k * (k - 1.0) / 2.0 / (1u64 << bits) as f64
}

/// Build record number `index`: header ‖ counter ‖ cookie.
/// Own-session records (every `OWN_INTERVAL`-th) carry the known zero cookie;
/// victim cookies are derived from the counter.
fn record(index: usize) -> [u8; 16] {
    let mut block = [0u8; 16];
    block[..8].copy_from_slice(&HEADER);
    block[8..12].copy_from_slice(&(index as u32).to_le_bytes());
    let cookie = if is_own(index) {
        [0u8; 4] // attacker's own session: known plaintext
    } else {
        (index as u32).wrapping_mul(0x9e37_79b9).to_le_bytes()
    };
    block[12..].copy_from_slice(&cookie);
    block
}

/// Is record `index` one of the attacker's own (known-cookie) records?
fn is_own(index: usize) -> bool {
    index.is_multiple_of(OWN_INTERVAL)
}

/// The truncated block value of a record: AES-128(record) kept to `bits` bits.
fn block_value(cipher: &Aes128, record: &[u8; 16], bits: u32) -> u32 {
    let mut block = Block::default();
    block.copy_from_slice(record);
    cipher.encrypt_block(&mut block);
    let bytes = block.as_slice();
    // Low bits of the first 4 bytes (little-endian read), masked to the space.
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) & ((1u32 << bits) - 1)
}

/// The exploitable collision the attack waits for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collision {
    /// Total records sent when the collision was found.
    pub blocks_sent: usize,
    /// The colliding truncated block value.
    pub block_value: u32,
    /// Record index of the victim (secret-cookie) record.
    pub victim_index: usize,
    /// Record index of the attacker's own (known-cookie) record.
    pub own_index: usize,
    /// The victim's cookie, recovered by XORing the two records.
    pub leaked_cookie: [u8; 4],
}

/// Streaming birthday-attack simulator. Deterministic: no RNG, everything
/// derives from record counters and the fixed demo key.
///
/// `PartialEq`/`Debug` compare the observable attack state, not the cipher
/// (the demo key is a constant, so equal state implies equal cipher).
#[derive(Clone)]
pub struct Simulator {
    bits: u32,
    cipher: Aes128,
    seen: HashMap<u32, usize>,
    /// Total colliding pairs observed (any pair, exploitable or not).
    pub collisions: usize,
    pub records_sent: usize,
    pub exploit: Option<Collision>,
}

impl PartialEq for Simulator {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
            && self.seen == other.seen
            && self.collisions == other.collisions
            && self.records_sent == other.records_sent
            && self.exploit == other.exploit
    }
}

impl std::fmt::Debug for Simulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Simulator")
            .field("bits", &self.bits)
            .field("collisions", &self.collisions)
            .field("records_sent", &self.records_sent)
            .field("exploit", &self.exploit)
            .finish_non_exhaustive()
    }
}

impl Simulator {
    pub fn new(bits: u32) -> Self {
        Self {
            bits,
            cipher: Aes128::new(&Block::from(DEMO_KEY)),
            seen: HashMap::new(),
            collisions: 0,
            records_sent: 0,
            exploit: None,
        }
    }

    /// The demo configuration the modes run.
    pub fn demo() -> Self {
        Self::new(DEMO_BLOCK_BITS)
    }

    /// Truncated block values of the first `n` records, without consuming
    /// the stream — for previews.
    pub fn preview_blocks(&self, n: usize) -> Vec<u32> {
        (0..n)
            .map(|i| block_value(&self.cipher, &record(i), self.bits))
            .collect()
    }

    /// Send up to `n` more records, stopping early at the exploitable
    /// collision. Returns how many records were actually sent.
    pub fn step(&mut self, n: usize) -> usize {
        let mut sent = 0;
        while sent < n && self.exploit.is_none() {
            self.send_one();
            sent += 1;
        }
        sent
    }

    /// Send exactly `n` records, recording (but not stopping at) the
    /// exploitable collision — for observing accumulated collision counts at
    /// a fixed block count.
    pub fn step_past_exploit(&mut self, n: usize) {
        for _ in 0..n {
            self.send_one();
        }
    }

    /// Send one record, updating collision state.
    fn send_one(&mut self) {
        let index = self.records_sent;
        let current = record(index);
        let value = block_value(&self.cipher, &current, self.bits);
        match self.seen.get(&value) {
            Some(&other) => {
                self.collisions += 1;
                // An own/victim pair is exploitable: XOR cancels the
                // identical header and the known counters, and the own
                // cookie is zero, so what remains IS the victim cookie.
                if is_own(index) != is_own(other) {
                    let (victim, own) = if is_own(index) {
                        (other, index)
                    } else {
                        (index, other)
                    };
                    let victim_record = record(victim);
                    let own_record = record(own);
                    let mut leaked = [0u8; 4];
                    for i in 0..4 {
                        leaked[i] = victim_record[12 + i] ^ own_record[12 + i];
                    }
                    self.exploit = Some(Collision {
                        blocks_sent: self.records_sent + 1,
                        block_value: value,
                        victim_index: victim,
                        own_index: own,
                        leaked_cookie: leaked,
                    });
                }
            }
            None => {
                self.seen.insert(value, index);
            }
        }
        self.records_sent += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run the demo simulator to its exploitable collision.
    fn run_to_exploit() -> Simulator {
        let mut sim = Simulator::demo();
        while sim.exploit.is_none() && sim.records_sent < 100_000 {
            sim.step(1024);
        }
        assert!(
            sim.exploit.is_some(),
            "demo space must collide well before 100k blocks"
        );
        sim
    }

    #[test]
    fn demo_terminates_quickly_enough_to_watch() {
        let sim = run_to_exploit();
        let sent = sim.exploit.unwrap().blocks_sent;
        // The fixed construction's draw lands at 4744; pin a generous
        // absolute bound so pace regressions surface here, not in the UI.
        assert!(sent < 8000, "demo exploit took {sent} blocks");
    }

    #[test]
    fn collision_count_scales_with_the_space_size() {
        // The birthday law in observable form: after a FIXED number of
        // blocks, collision counts scale like 1/space. Halving the space
        // 16× should multiply collisions ~16×. First-collision positions
        // are single exponential draws (heavy tails) and cannot be compared
        // directly; accumulated counts average the noise out.
        let collisions_after = |bits: u32, blocks: usize| {
            let mut sim = Simulator::new(bits);
            sim.step_past_exploit(blocks);
            sim.collisions
        };
        // 6000 blocks: expected counts ≈ 275 (16-bit) and ≈ 17 (20-bit),
        // both far from zero, so the ratio is meaningful.
        let small_space = collisions_after(16, 6000) as f64;
        let large_space = collisions_after(20, 6000) as f64;
        let ratio = small_space / large_space;
        assert!(
            (8.0..=32.0).contains(&ratio),
            "16-bit/20-bit collision ratio {ratio} not near 16 (small={small_space}, large={large_space})"
        );
    }

    #[test]
    fn leak_recovers_the_victims_true_cookie() {
        let sim = run_to_exploit();
        let exploit = sim.exploit.unwrap();
        let victim_record = record(exploit.victim_index);
        let mut true_cookie = [0u8; 4];
        true_cookie.copy_from_slice(&victim_record[12..]);
        assert_eq!(exploit.leaked_cookie, true_cookie);
        assert_ne!(
            exploit.leaked_cookie, [0u8; 4],
            "victim cookie is non-trivial"
        );
        // And the pair really is one own + one victim record.
        assert!(is_own(exploit.own_index));
        assert!(!is_own(exploit.victim_index));
    }

    #[test]
    fn expected_curve_brackets_the_observed_collision_count() {
        let sim = run_to_exploit();
        let observed = sim.collisions as f64;
        let expected = expected_collisions(sim.records_sent, DEMO_BLOCK_BITS);
        assert!(
            (0.25..=4.0).contains(&(observed / expected)),
            "observed {observed} vs expected {expected}"
        );
    }

    #[test]
    fn deterministic_across_runs() {
        let a = run_to_exploit();
        let b = run_to_exploit();
        assert_eq!(a.records_sent, b.records_sent);
        assert_eq!(a.exploit, b.exploit);
    }

    #[test]
    fn block_values_stay_inside_the_truncated_space() {
        let sim = Simulator::new(12);
        for index in 0..100 {
            let value = block_value(&sim.cipher, &record(index), 12);
            assert!(value < (1 << 12));
        }
    }

    #[test]
    fn first_collision_expectation_matches_the_window_center() {
        // For the demo space: ~1281, i.e. ≈ 2^(n/2) with the birthday constant.
        let e = first_collision_expectation(DEMO_BLOCK_BITS);
        assert!((1100.0..1500.0).contains(&e), "got {e}");
    }
}
