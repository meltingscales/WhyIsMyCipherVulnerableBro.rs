# WhyIsMyCipherVulnerableBro.rs

> "Why is my cipher vulnerable, bro?" — find out, visually, in your terminal.

A Rust TUI (built with [ratatui](https://ratatui.rs)) for exploring the **math
behind vulnerable and "export-grade" ciphers** — the ones that quietly shipped
in TLS for a decade and then got absolutely cooked in public.

Watch the attacks actually happen: birthday collisions piling up, export keys
falling to factoring, fake intercepted traffic decrypting in real time.

## Attack gallery

| Attack  | What broke      | The math                                                          |
| ------- | --------------- | ----------------------------------------------------------------- |
| SWEET32 | 3DES in TLS     | Birthday paradox — around 2^32 blocks in, collisions leak cookies  |
| FREAK   | RSA_EXPORT keys | Factoring a 512-bit "export" modulus nobody should have ever used  |
| Logjam  | DHE_EXPORT      | Discrete log in a 512-bit group, plus a downgrade to get you there |

More to come — see the [roadmap](#status).

## Modes

- **Autocrack** — pick a victim cipher, watch the attack run with cool visuals
- **Traffic simulator** — pretend to intercept fake web traffic, then decrypt it
- **Lessons** — cryptopals-style, step-by-step walk-throughs of why each cipher dies
- **Math views** — graphs and matrices directly tied to the underlying math, not just pretty ASCII

## Status

Pre-alpha — currently a skeleton. Roadmap:

- [x] Core TUI framework + navigation
- [x] SWEET32 demo
- [ ] FREAK demo
- [ ] Logjam demo
- [ ] Traffic simulator mode
- [ ] Autocrack mode with visuals
- [ ] Cryptopals-style lessons
- [ ] More fun-to-watch vulnerabilities with interesting math (suggestions welcome)

## Getting started

Requires Rust (2024 edition); [just](https://github.com/casey/just) is optional.

```sh
cargo run    # or: just run
```

## Development

Changes are planned with [OpenSpec](https://openspec.dev) — see `openspec/`.

AI-assisted development is done with [z.ai](https://z.ai) (GLM) — it doesn't
refuse legitimate crypto-education content.

## Why though?

Because reading "the birthday bound is 2^(n/2)" in a textbook is one thing, and
watching a counter tick toward it is another. Everything here attacks
**simulated, local victims only** — this is a teaching tool for people who want
to understand why today's TLS config should not contain yesterday's mistakes.
