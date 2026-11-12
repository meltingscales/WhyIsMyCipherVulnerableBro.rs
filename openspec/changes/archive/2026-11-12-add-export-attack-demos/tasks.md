# Tasks

## 1. Arithmetic core (pure cipher modules)

- [x] 1.1 Add a shared `u64` mulmod/powmod helper in `src/cipher/mod.rs` (tested against known values, incl. overflow-safe paths); verify with unit tests
- [x] 1.2 Create `src/cipher/rsa.rs` — fixed ~32-bit primes, 64-bit modulus, e = 65537 keygen with d = e⁻¹ mod λ, encrypt/decrypt of a short secret, and step-counted Pollard's rho (`step(n)` API, fixed seeds, iteration counter); verify with unit tests: round-trip encryption, rho recovers both fixed primes, iteration count within [N^¼/2, 4·N^¼], and the recovered key decrypts the secret (full kill chain)
- [x] 1.3 Create `src/cipher/dlog.rs` — fixed ~40-bit safe prime with generator, DH exchange values, and step-counted baby-step giant-step (`step(n)` API, baby-then-giant phases, step counter); verify with unit tests: the recovered exponent reproduces the public value, the step count is within [√N/2, 2·√N], and the shared secret decrypts the session bytes (full kill chain)

## 2. Autocrack victims

- [x] 2.1 Add FREAK and Logjam victims and their attack kinds to `src/modes/autocrack.rs` — downgrade narration rows first, then solver progress rows per tick (rho iterations / BSGS steps), done state showing the full chain (FREAK: iterations, primes, private exponent, secret; Logjam: steps, exponent, shared secret, session bytes); update victim blurbs and the `VICTIMS` table; verify with unit tests (both runs complete via `advance` within a tick budget, downgrade rows precede attack rows, done fields populated) and `TestBackend` tests (victims listed, running shows solver counters, done shows each chain's results)

## 3. Lessons

- [x] 3.1 Author lesson four ("FREAK — when 'export grade' met factoring": export-crypto history, the 512-bit modulus, factoring economics, precomputation amortization, the downgrade bug) and lesson five ("Logjam — discrete logs and the prime everyone shared": DH in two pictures, BSGS math, why 512-bit groups fell, the shared-prime precomputation twist, the shared moral) in `src/modes/lessons.rs`, citing the simulators' pinned step counts; verify with unit tests that both open, step through, and cite the pinned numbers

## 4. Math Views attack-cost view

- [x] 4.1 Make `v` cycle three views and add the attack-cost view (observed vs expected steps: rho N^¼ at two sizes, BSGS √N at two sizes, brute-force reference in the caption, all labeled) in `src/modes/mathviews.rs`, with observed data from OnceLock-run solvers; verify with unit tests for the three-state cycle and `TestBackend` tests that the cost view renders both series labeled

## 5. Integration and verification

- [x] 5.1 Extend the MCP harness integration test with FREAK and Logjam scenarios (select victim, tick to completion, snapshot shows each chain's recovered results) and tick the README's `FREAK demo` and `Logjam demo` checkboxes; verify by running the integration test and the README rendering
- [x] 5.2 Full pass: `cargo test`, `just lint`, `cargo fmt --check` clean; `--mcp` scripted sessions walking both attacks to completion

## Workflow follow-up

- Archive the change after implementation is verified.
- Optional follow-up: 3DES/DHE_EXPORT sessions in the Traffic Simulator; a combined "2015: the export-crypto reckoning" umbrella lesson linking SWEET32/FREAK/Logjam.
