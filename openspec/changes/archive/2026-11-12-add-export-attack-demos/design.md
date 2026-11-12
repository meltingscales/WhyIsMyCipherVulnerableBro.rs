# Design

## Context

See `proposal.md` — Why. The SWEET32 change established the real-attack pattern: pure cipher-core module with counted-step solvers, autocrack victim streaming progress rows, lesson citing the core's pinned numbers, math-views observed-vs-expected chart, MCP harness scenario in the integration test. This change runs that pattern twice.

## Goals / Non-Goals

**Goals:**

- FREAK's full kill chain live: downgrade → factor → derive private key → decrypt.
- Logjam's full kill chain live: downgrade → discrete log → shared secret → decrypt.
- Both solvers count their steps, so the cost math (N^¼, √N) is observable, not asserted.
- One lesson arc connecting both: export-grade crypto was a deliberate vulnerability with a deadline.

**Non-Goals:**

- Real parameter sizes (512-bit factoring/dlog are nation-state-weeks; the toys run in seconds and the lessons say so).
- TLS state-machine fidelity — the downgrade is narrated rows (ClientHello → MITM strip → ServerHello), not byte-level messages.
- General RSA/DH libraries — u64 arithmetic, fixed deterministic parameters.

## Decisions

**1. Two cipher modules, not one.**
`cipher/rsa` (keygen from fixed primes, modpow, rho factoring) and `cipher/dlog` (safe-prime group, DH exchange, BSGS) share no code beyond u64 mulmod helpers — separate files keep each lesson's math self-contained. A tiny shared `u64_powmod` lives in `cipher/mod.rs`.

**2. Deterministic parameters, chosen once.**
- RSA: two fixed ~32-bit primes (picked so rho takes a watchable number of iterations, target ~50-100k) → 64-bit modulus. Same primes every run; tests pin the iteration count to a band.
- DH: fixed ~40-bit safe prime p (p = 2q+1 with q prime), generator 5, fixed server exponent; the client exponent is the attack target. BSGS on a 40-bit group needs ≤ ~2^20 steps — streams at 65k steps/tick over ~16 ticks.

**3. Streamed solvers, step-counted.**
Both solvers expose `step(n)`-style APIs like the birthday simulator: rho returns after n iterations (or on factor); BSGS extends baby steps (then giant steps) by n per call. Both report counters the UI streams; both deterministic (rho's polynomial walk from fixed seeds — no RNG).

**4. Downgrade theater as phase-1 rows.**
Each run's first rows narrate the exchange (fixed 3-4 rows), then the attack rows take over. This is presentation, honestly labeled — the lesson explains the real MITM needed position + a TLS state-machine bug (FREAK) / the server's acceptance of the export suite (Logjam).

**5. FREAK recovery mechanics (toy-faithful).**
Factor N → p, q → λ = lcm(p−1, q−1) → d = e⁻¹ mod λ → decrypt the "premaster secret" ciphertext (e = 65537). The done screen shows primes, d, and the secret. Real FREAK: same chain on a 512-bit N where factoring was precomputed once and amortized across servers — the lesson tells that part.

**6. Logjam recovery mechanics (toy-faithful).**
Public: p, g, server public value Y = g^y. BSGS finds y (the server's exponent is the demo's target). Shared secret = Y^client_exp... toy simplification: recovering ANY private exponent that reproduces Y yields the session key derived from the DH shared secret (hash-to-key, demo uses the shared secret bytes directly). Done screen shows y, the shared secret, decrypted bytes.

**7. Math-views attack-cost chart.**
Third view in the `v` cycle: bar pairs (observed, expected) for rho at two modulus sizes and BSGS at two group sizes, expected = N^¼ (rho, N = smaller factor) and √N (BSGS), plus a brute-force reference line in the caption. Observed values come from running the actual solvers once (OnceLock, like the birthday checkpoints).

**8. Pacing.**
RHO_STRIDE = 4096 iterations/tick (target ≈ 15-25 ticks); BSGS_STRIDE = 65_536 steps/tick (target ≈ 16 ticks). Both named constants.

## Risks / Trade-offs

- [Rho iteration count is a draw (like the birthday collision)] → Fixed parameters make it deterministic; tests pin a band [N^¼/2, 4·N^¼] and the exact count goes into the lesson prose. If the chosen prime pair lands outside the band, pick different fixed primes — that's parameter choice, not test fudging.
- [BSGS baby-step table memory] → 2^20 entries × 8 bytes ≈ 8 MB, built incrementally; acceptable for a TUI, freed on completion.
- [Downgrade rows may read as "fake content"] → They are labeled narration; the lessons explicitly describe the real mechanism. Honest framing beats a fake protocol replay.
- [Three views on one `v` key] → Spec scenario pins the cycle order; footer hint says "v view".

## Migration Plan

Additive; no existing behavior removed (victim-selection wording extends the "at minimum" list; completion wording generalizes; birthday-view requirement gains the cycle). One commit series; rollback is `git revert`. README ticks both roadmap boxes on merge.
