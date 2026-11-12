# Proposal

## Why

SWEET32 proved the pattern: a real named attack, scaled until its math is watchable, lands as cipher-core module + autocrack victim + lesson + math view. FREAK and Logjam are next on the README roadmap and they share one historical story — the 2015 export-grade downgrade attacks — while teaching two different mathematics: FREAK dies to **integer factoring** (factor the 512-bit RSA_EXPORT modulus, derive the private key), Logjam dies to **discrete logarithms** (solve dlog in the 512-bit DHE_EXPORT group, recover the shared secret). One change, two attacks, one "why export crypto was a time bomb" arc.

## What Changes

- New pure module `cipher/rsa`: toy RSA — deterministic keygen from fixed primes, encrypt/decrypt of a short premaster secret, and live factoring of the modulus via Pollard's rho with an iteration counter (recovering the primes reconstitutes the private key, exactly FREAK's kill chain).
- New pure module `cipher/dlog`: toy Diffie–Hellman in a fixed small safe-prime group, a full exchange, and discrete-log recovery via baby-step giant-step with a step counter (recovering the exponent yields the shared secret, exactly Logjam's kill chain).
- Autocrack gains two victims:
  - **FREAK (export RSA)** — streams the downgrade theater (client offers strong, MITM strips to export), then rho iterations per tick until the ~64-bit toy modulus factors; done state shows the primes, the derived private key, and the decrypted secret.
  - **Logjam (export DHE)** — streams the downgrade, then BSGS steps per tick until the discrete log lands; done state shows the recovered exponent, the shared secret, and the decrypted session bytes.
- Lessons ships two new lessons: "FREAK — when 'export grade' met factoring" and "Logjam — discrete logs and the prime everyone shared", closing with the shared moral (downgrade + precomputation).
- Math Views: `v` now cycles **three** views — frequency, birthday, and a new **attack-cost** view charting observed attack steps vs the expected complexity curves (√N birthday, N^¼ rho/BSGS) at several problem sizes.
- Autocrack's completion requirement generalizes from "birthday attacks" to structural attacks (collisions, factorizations, discrete logs).

Scale honesty, like SWEET32's truncated AES: real FREAK factored a 512-bit modulus (weeks on big iron, or precomputed); the demo's toy modulus factors live in seconds. The lesson says so explicitly.

Not included: real RSA/DH parameter sizes, a MITM traffic view in Traffic Simulator (follow-up nicety), TLS message parsing (the downgrade is narrated rows, not a protocol replay).

## Capabilities

### New Capabilities

(none — simulator mechanics stay internal; observable behavior is specified through the modified modes)

### Modified Capabilities

- `autocrack`: victim list gains FREAK and Logjam; completion generalizes to structural-attack results; ADDED requirements for each new victim's streamed attack and outcome.
- `lessons`: shipped-lessons requirement raised to five (Caesar, XOR, SWEET32, FREAK, Logjam).
- `math-views`: `v` cycles three views; ADDED attack-cost view requirement.

## Impact

- **Code**: new `src/cipher/rsa.rs`, `src/cipher/dlog.rs`; `src/modes/autocrack.rs` (two victims + attack kinds + downgrade phase rows), `src/modes/lessons.rs` (two lessons), `src/modes/mathviews.rs` (third view, cycling).
- **Dependencies**: none new — pure arithmetic on u64.
- **Testing**: cipher-core tests pin rho factoring correctness and iteration counts within the N^¼ band, BSGS correctness and the √N step band, and full kill-chain recovery (primes → private key → plaintext; exponent → shared secret → plaintext). The MCP harness integration test gains FREAK and Logjam scenarios.
- **Docs**: README roadmap ticks `FREAK demo` and `Logjam demo`.
