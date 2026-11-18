# Proposal

## Why

The repo is public with zero automation: nothing verifies a change on the platforms the TUI actually targets (terminal behavior differs across Windows/macOS/Linux), and there is no way to hand someone a runnable binary without asking them to install a Rust toolchain. CI makes every future change — starting with `add-core-tui-framework` — land verified on all three OSes, and tag-triggered releases publish downloadable binaries for the three platforms.

## What Changes

- Add a CI workflow (`.github/workflows/ci.yml`): on pushes to `main` and on PRs, run `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` across an ubuntu/windows/macos matrix.
- Add a release workflow (`.github/workflows/release.yml`): on `v*` tag push (plus manual dispatch), build optimized binaries for Linux x86_64, macOS x86_64 + aarch64, and Windows x86_64, package each as an archive with the README, and attach them to the tag's GitHub release.
- README: add CI badge and a short "Downloads" note pointing at the releases page.

Assumptions (minor, recorded): CI triggers are push/PR to `main` only; releases cut on `v*` tags; CI calls cargo directly (no `justfile` dependency — `add-core-tui-framework`, which adds the justfile, is not yet applied). The repo currently has **no LICENSE**, so release archives ship without one until a license change lands.

## Capabilities

### New Capabilities

- `ci-cd`: The project's automation pipeline — continuous-integration checks on pushes and PRs, and the tag-triggered release pipeline that builds and publishes per-platform binaries.

### Modified Capabilities

(none — no existing specs)

## Impact

- **Code**: none — workflows and docs only.
- **Repo layout**: new `.github/workflows/ci.yml`, `.github/workflows/release.yml`; README gains a badge and a downloads section.
- **Dependencies/tooling**: GitHub Actions runners (ubuntu/windows/macos); no new crate dependencies; consumers need no Rust toolchain to run release binaries.
- **Known gap surfaced**: public binaries without a LICENSE — worth a follow-up change, not blocking this one.
