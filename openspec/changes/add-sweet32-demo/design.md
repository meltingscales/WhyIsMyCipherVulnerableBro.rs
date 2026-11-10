# Design

## Context

See `proposal.md` — Why. The cipher core is pure and TUI-free with per-module known-vector tests; autocrack streams attack rows via a pure `advance` step; math-views renders charts from cipher-core data; the MCP harness verifies scenarios without a terminal. `aes` 0.9 sits unused in `Cargo.toml`.

## Goals / Non-Goals

**Goals:**

- A real, named attack (SWEET32) whose full lesson is the birthday bound, watchable end to end in seconds, not hours.
- The `aes` dependency earning its place as the demo's block function.
- The pattern for FREAK/Logjam follow-ups: cipher-core module + victim + lesson + math view.

**Non-Goals:**

- Actual 3DES (truncated AES *is* the point), real TLS parsing, multi-collision/long-run statistics beyond first collision.
- FREAK/Logjam; traffic-simulator sessions with small-block ciphers.

## Decisions

**1. Block function: AES-ECB single block, truncated to n bits.**
Encrypt a 16-byte record block under a fixed demo key and keep the low `n` bits as the block value (demo default n = 20 → space of 2^20). Why: (a) uses the shipped crate; (b) it makes the lesson airtight — the attack never touches the cipher's strength, only the block space, so "even AES dies" is demonstrated, not asserted. Real SWEET32: n = 64, ~2^32 blocks; the demo's n is just a scale factor on the same curve.

**2. Demo block space n = 20, stride 96 blocks per tick.**
Expected first collision ≈ √(π/2 · 2^n) ≈ 1 281 blocks for n = 20 → about 13 ticks at 96 blocks/tick ≈ 3 s on screen, comfortably inside a single viewing session. Both numbers are named constants, tunable without touching logic.

**3. Records carry structure so the leak is real.**
Each simulated record = known header bytes + secret cookie bytes, built deterministically from a record counter (no RNG — tests and reruns are identical). Block = truncated AES(header ‖ counter). At the first collision, the attacker XORs the two colliding records' known-plaintext structure to expose cookie material — the same known-plaintext reasoning as the real attack, at demo scale.

**4. Autocrack attack-kind enum, stats rows instead of candidate rows.**
`Run` grows an attack kind (Classical vs Birthday). Birthday runs stream one summary row per tick (blocks sent, collisions, expected count), plus an event row when the collision lands; completion shows collision count vs birthday bound and the leaked material. The existing "best row" highlight is meaningless for birthdays and is skipped for that kind — the spec's completion requirement is generalized accordingly.

**5. Math-views birthday view from the same simulator.**
`v` toggles chart kinds. The birthday chart renders the simulator run for the demo space: observed collisions per checkpoint (fixed checkpoints, e.g. every 128 blocks) and the expected curve E[collisions] ≈ k²/2^(n+1), as two labeled series over blocks sent. `BarChart` with two groups? No — keep to one `Chart`-free approach: a bar chart of observed collisions with the expected value printed per checkpoint row (ratatui `Chart` plotting is available but the bar+number form reads better at 80 cols and matches the frequency view's idiom).

**6. Lesson three mirrors the simulator's numbers.**
Steps: birthday problem intuition → the 2^(n/2) bound arithmetic → 64-bit blocks = 32 GB of traffic (why 2016 killed 3DES) → the demo's truncated-AES honesty note → the known-plaintext leak mechanism → "watch it in Autocrack".

## Risks / Trade-offs

- [Demo collision lands far from the mean (heavy tail)] → The unit test pins the *window* (first collision within [2^(n/2)/2, 3·2^(n/2)] for the fixed seed — deterministic given no RNG, so the bound is stable; if the seed ever drifts, the test says so loudly.
- [Stats rows read as "less exciting" than candidate streams] → The collision event row and the leak reveal are the payoff; pacing constants keep the wait under 5 s.
- [Two chart kinds complicate math-views state] → One enum field + a `v` handler; footer hint string gains `v view`.
- [Truncation misconstrued as "AES is broken"] → Lesson step and autocrack blurb both state the block-size-is-the-lesson point explicitly.

## Migration Plan

Additive to the modes; no existing behavior removed (selection-wording fix aligns spec with shipped behavior). One commit series behind the change; rollback is `git revert`. README roadmap checkbox ticks on merge.
