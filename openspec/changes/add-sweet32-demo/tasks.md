# Tasks

## 1. Birthday simulator (pure cipher core)

- [ ] 1.1 Create `src/cipher/sweet32.rs` — deterministic record generator (known header + cookie bytes + counter), AES-ECB single-block encryption truncated to the demo block space (n = 20), collision tracking across records, first-collision leak via XOR of the colliding records' structure, expected-collision function E ≈ k²/2^(n+1), tunable constants (block bits, tick stride); verify with unit tests: determinism (same seed → same collision index), the first collision lands inside [2^(n/2)/2, 3·2^(n/2)] for the fixed seed, and the leak exposes the cookie bytes
- [ ] 1.2 Verify the demo end-to-end at the pure layer: streaming the simulator for the demo space terminates with a collision under 4 000 blocks and the expected-collision function brackets the observed count at collision; record the numbers in the test for the lesson to cite

## 2. Autocrack victim

- [ ] 2.1 Add the SWEET32 victim and attack-kind enum to `src/modes/autocrack.rs` — birthday runs stream one stats row per tick (blocks sent, collisions, expected), an event row at the collision, and a done state showing collision count vs birthday bound plus the leaked cookie material; update the victim-list rendering and blurb for the new victim; verify with unit tests (run to completion via `advance`, stats rows count, done carries collision + leak) and `TestBackend` tests (victim listed, running shows the three counters, done shows the bound comparison and leak)

## 3. Lesson

- [ ] 3.1 Author lesson three ("SWEET32 — the birthday problem kills 3DES") in `src/modes/lessons.rs` — birthday intuition, the 2^(n/2) bound, 64-bit blocks in 2016, the truncated-AES honesty note, the known-plaintext leak; verify with unit tests that the lesson opens, steps, and cites the simulator's pinned numbers (collision window from task 1.2)

## 4. Math Views birthday chart

- [ ] 4.1 Add the birthday-collision view to `src/modes/mathviews.rs` — `v` toggles between the frequency chart and the birthday chart (observed collisions per checkpoint plus the expected value, both labeled, view named), footer hint gains `v view`; verify with unit tests for toggle state and wrap behavior of existing arrows (unchanged), and `TestBackend` tests that both views render their labels

## 5. Integration and verification

- [ ] 5.1 Extend the MCP harness integration test with a SWEET32 scenario (select third victim, tick to completion, snapshot shows the collision bound and leak) and tick the README roadmap's "SWEET32 demo" checkbox; verify by running the integration test and checking the README renders
- [ ] 5.2 Full pass: `cargo test`, `just lint`, `cargo fmt --check` clean; a `--mcp` scripted session walking SWEET32 to completion (replacing the PTY emulator for this change entirely)

## Workflow follow-up

- Archive the change after implementation is verified.
- FREAK and Logjam follow this same shape (cipher-core module + victim + lesson + math view); propose each separately.
