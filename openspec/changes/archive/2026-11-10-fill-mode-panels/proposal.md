# Proposal

## Why

The TUI shell navigates between four screens that all say "not implemented yet." Every demo, lesson, and visualization this project exists for now has nowhere to land. Filling each panel with a working starter slice — real cipher logic, real interaction, real visuals — turns the shell into the teaching tool the README promises, and establishes the screen architecture and pure cipher core that SWEET32/FREAK/Logjam changes will extend.

## What Changes

Each of the four modes gets its real layout and a working first feature, built on classic teaching ciphers (the same ones cryptopals starts with):

- **Autocrack**: pick a victim cipher (Caesar or Vigenère) from a list, watch the brute-force/frequency attack run live, tick by tick, with per-candidate scores and the best key called out.
- **Traffic Simulator**: a list of fake intercepted sessions (built-in, fixed templates); select one, inspect its hex dump, then run the known-plaintext/repeating-XOR break and watch the plaintext reveal.
- **Lessons**: lesson list plus a step-by-step reader; ships with two lessons matching the implemented attacks — "Why Caesar dies" (frequency analysis / chi-squared) and "Breaking repeating-XOR".
- **Math Views**: a letter-frequency bar chart comparing plaintext vs ciphertext samples, with the chi-squared and index-of-coincidence numbers that make the attack work.

Supporting changes:

- Cipher logic lives in a pure, TUI-free `cipher` module (Caesar, Vigenère, repeating-XOR, frequency statistics) with unit tests against known vectors.
- Screen architecture: per-mode modules owning their own state, key handling, and rendering, routed by the shell (this revises the tui-shell design note that deferred the abstraction — see design).
- Footer shows the active mode's keybindings in addition to the global ones.
- **BREAKING** (spec-level): removes `tui-shell`'s "Placeholder mode screens" requirement — superseded by the per-mode content specs below.

Not included: SWEET32/FREAK/Logjam, user-supplied ciphertext input, editing/persistence, scrolling lesson bodies beyond a step.

## Capabilities

### New Capabilities

- `autocrack`: The attack-runner mode — victim selection, live tick-driven attack execution with scores, reset and re-run.
- `traffic-simulator`: The interception mode — fake session list, hex inspection, and the decrypt action that reveals plaintext.
- `lessons`: The guided-reading mode — lesson selection and step-by-step navigation over authored content.
- `math-views`: The visualization mode — frequency charts and the statistics that justify the attacks.

### Modified Capabilities

- `tui-shell`: REMOVED requirement "Placeholder mode screens" — real per-mode content now owns what each screen renders; no other shell behavior changes (lifecycle, navigation, tab bar, footer remain).

## Impact

- **Code**: new `src/cipher/` module; new `src/modes/` modules (autocrack, traffic-simulator, lessons, math-views); `ui.rs` placeholder rendering replaced by per-mode dispatch; `event.rs` gains mode-key routing (global keys take precedence); `App` grows per-mode state.
- **Dependencies**: none new — `aes` remains unused until a real demo needs it.
- **Docs**: README gains no new claims (roadmap items for SWEET32/FREAK/Logjam stay open); lessons content is authored in-repo.
- **Specs**: `tui-shell` placeholder requirement removed; four new capability specs become the contracts for mode content.
