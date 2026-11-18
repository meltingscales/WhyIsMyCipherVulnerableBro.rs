# Design

## Context

See `proposal.md` — Why. Current state: public repo, no `.github/` directory, `Cargo.lock` committed, deps are pure Rust (`ratatui`, `crossterm`, `aes` — no C toolchains), and the one in-flight change (`add-core-tui-framework`) is unapplied, so only cargo-direct commands can be assumed to exist.

## Goals / Non-Goals

**Goals:**

- Every push/PR verified on all three OSes the app targets.
- One-command release: push a `v*` tag, get a GitHub release with runnable binaries per platform.
- This change stands alone — no dependency on `add-core-tui-framework` being applied first.

**Non-Goals:**

- Branch protection rules, required-status checks (repo settings, not code — configure manually after).
- macOS universal binaries, Linux aarch64/musl, Windows aarch64 (add later if asked).
- Code signing/notarization, homebrew/scoop packaging, changelog generation beyond GitHub's auto-notes.

## Decisions

**1. Native runners per target, not cross-compilation.**
Release matrix: `ubuntu-latest` → x86_64-unknown-linux-gnu; `macos-13` → x86_64-apple-darwin; `macos-14` → aarch64-apple-darwin; `windows-latest` → x86_64-pc-windows-msvc. Alternative: one ubuntu runner with `cross` — fewer jobs but fragile toolchains for the mac/windows targets; native runners are the reliable default and free on public repos.

**2. Two workflows with separate triggers.**
`ci.yml` on `push: main` + `pull_request`; `release.yml` on `push: tags: v*` + `workflow_dispatch` (tag input). CI stays fast; releases stay explicit. Alternative: one workflow branching on trigger — harder to read, easier to misfire.

**3. Publish only after the full matrix succeeds.**
Release build matrix uploads workflow artifacts per target; a separate `publish` job (`needs: build`) downloads them all, creates the release with auto-generated notes, and attaches the archives in one step. This is what makes the "failed target blocks publication" scenario hold. Alternative: `softprops/action-gh-release` per matrix job — publishes partial releases when one target fails, and adds a third-party action dependency; prefer the preinstalled `gh` CLI with `GH_TOKEN`.

**4. Packaging conventions.**
Archives named `why_is_my_cipher_vulnerable_bro-v{version}-{target}.tar.gz` (zip on Windows, which names the binary `.exe`). Each contains the binary and `README.md`. Add `strip = true` to `[profile.release]` in `Cargo.toml` (the only file outside `.github/` and README this change touches — applies to local builds too, which is fine for this project). Builds run with `--locked` since `Cargo.lock` is committed.

**5. CI calls cargo directly, not `just`.**
The justfile lands with `add-core-tui-framework`, which may be applied before or after this change. Using bare cargo keeps the workflows order-independent; switching CI to `just` later is optional polish, not a requirement.

## Risks / Trade-offs

- [GitHub's `macos-latest` alias drifted to arm64 before] → Pin explicit runner labels (`macos-13`, `macos-14`) per target rather than trusting `latest`.
- [Untested workflow YAML fails on the first real tag] → Validate with `workflow_dispatch` against a throwaway tag before the first real release; CI path validates itself on the merge commit.
- [Public binaries with no LICENSE] → Flagged in the proposal; a follow-up change should add one and archives will then include it via the packaging step.
- [Official actions pinned only by major tag (`actions/checkout@v5`)] → Acceptable supply-chain posture for a public hobby repo; SHA-pinning can come later without structural change.

## Migration Plan

Add the two workflow files, the release-profile strip, and the README updates in one commit. Verify CI on the push itself; verify release via dispatch on a throwaway tag, then delete the throwaway release. Rollback: `git revert` and delete the tag's release — nothing persistent outside GitHub Actions.
