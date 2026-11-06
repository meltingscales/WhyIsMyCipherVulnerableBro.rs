# Tasks

## 1. CI workflow

- [ ] 1.1 Confirm the baseline is green locally: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` all pass on the current commit (CI will fail otherwise for reasons unrelated to the workflow files)
- [ ] 1.2 Create `.github/workflows/ci.yml`: `push: branches: [main]` + `pull_request` triggers, matrix over ubuntu/macos/windows, running `cargo fmt --check`, `cargo clippy --locked -D warnings`, `cargo test --locked`; verify with `actionlint` (if installed) or YAML parse, then confirm the run appears on the push that merges this change
- [ ] 1.3 Open the applying PR (or check the merge commit's run) and verify all three platforms report pass/fail on it, matching the `ci-cd` spec's PR scenario

## 2. Release workflow

- [ ] 2.1 Add `[profile.release] strip = true` to `Cargo.toml`; verify `cargo build --release --locked` succeeds locally and the produced binary is stripped
- [ ] 2.2 Create `.github/workflows/release.yml`: `push: tags: ['v*']` + `workflow_dispatch` with a tag input; pinned-runner matrix (ubuntu-latest→linux x86_64, macos-13→macos x86_64, macos-14→macos aarch64, windows-latest→windows x86_64) building `--release --locked`, uploading per-target workflow artifacts (binary + README, tar.gz except zip on Windows); a `publish` job gated on the whole matrix that downloads the artifacts and attaches them to the tag's release via `gh release` with `GH_TOKEN`; verify with `actionlint` (if installed) or YAML parse
- [ ] 2.3 Dry-run the release path: push a throwaway tag (e.g. `v0.0.0-ci-test`), confirm the workflow publishes a release with all four archives, download two of them and confirm the binaries start, then delete the release and tag — satisfying the "tag push produces a complete release" and "runnable on stock systems" scenarios

## 3. Docs

- [ ] 3.1 Update README: CI status badge for `main` and a Downloads section pointing at the releases page; verify the badge renders and the releases link resolves

## 4. Integration verification

- [ ] 4.1 After this change lands on `main`: confirm the CI run is green on all three platforms, and re-check the release dry-run artifacts one final time against every `ci-cd` spec scenario

## Workflow follow-up

- Cut the first real release (`v0.1.0`) once the user decides the app is worth tagging — the workflow is ready before then.
- Consider a follow-up change adding a LICENSE so release archives ship legally unambiguous.
- Archive the change after implementation is verified.
