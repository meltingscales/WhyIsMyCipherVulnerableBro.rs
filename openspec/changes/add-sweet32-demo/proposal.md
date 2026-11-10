# Proposal

## Why

The app teaches with toy ciphers so far; the README's promise — SWEET32, FREAK, Logjam — is still open road. SWEET32 is the right first attack: its whole point is a probability bound (birthday collisions after ~2^(n/2) blocks in an n-bit block cipher), which is exactly the kind of math this app exists to make watchable. It also finally uses the `aes` crate: the demo encrypts with AES but truncates blocks to a demo-sized space, making the lesson sharper — SWEET32 killed 3DES for its *block size*, not its algorithm, and even truncated AES dies the same way.

## What Changes

- New pure module `cipher/sweet32`: a birthday-collision simulator — deterministic seeded "TLS records" (known header + secret cookie bits) encrypted as single AES blocks truncated to a configurable block space; collision detection; on first collision, the XOR of the colliding records recovers cookie material (the known-plaintext leak that is SWEET32's actual impact).
- Autocrack gains a third victim, **SWEET32 (birthday attack)**: the run streams blocks-sent, collisions-found, and the expected-collision count from the birthday math, ending at the first collision with the leaked cookie bits as the recovered result.
- Lessons ships a third lesson, "SWEET32 — the birthday problem kills 3DES" (birthday math, why 2^32 blocks killed 64-bit ciphers in 2016, and the truncated-AES honesty note).
- Math Views gains a birthday view: collisions observed vs blocks sent against the expected birthday curve, toggled with `v` alongside the existing frequency chart (footer hints update).
- Autocrack's "victim selection" and "completion" requirements are generalized: selection is arrow keys + Enter (bare `1`-`4` are the shell's, per `tui-shell`), and completion shows the attack's recovered result — key + plaintext for cipher breaks, colliding blocks + leaked material for birthday attacks.

Not included: FREAK and Logjam (follow-up changes of the same shape), real 3DES (the truncated-AES construction *is* the demonstration), traffic-simulator integration (a 3DES session there is a later nicety), and full birthday-curve math views beyond the collision count.

## Capabilities

### New Capabilities

(none — the simulator is internal mechanics; its observable behavior is specified through the three modified modes)

### Modified Capabilities

- `autocrack`: victim list gains SWEET32; selection wording corrected to arrows+Enter; completion generalized to per-attack results; new requirement for the birthday attack's streamed progress and collision outcome.
- `lessons`: shipped-lessons requirement raised to three (Caesar frequency, repeating-XOR, SWEET32 birthday).
- `math-views`: new birthday-collision view requirement with `v` toggle; existing frequency view unchanged.

## Impact

- **Code**: new `src/cipher/sweet32.rs`; `src/modes/autocrack.rs` (third victim, attack-kind enum, stats rows), `src/modes/lessons.rs` (third lesson), `src/modes/mathviews.rs` (birthday view + toggle); footer hints.
- **Dependencies**: none new — `aes` 0.9 finally used (AES-ECB single-block as the demo's block function).
- **Testing**: cipher-core unit tests pin the birthday window (first collision lands within the theoretical √ band), leak recovery, and determinism; mode tests as before; the MCP harness gains a SWEET32 scenario in its integration test.
- **Docs**: README roadmap ticks "SWEET32 demo".
