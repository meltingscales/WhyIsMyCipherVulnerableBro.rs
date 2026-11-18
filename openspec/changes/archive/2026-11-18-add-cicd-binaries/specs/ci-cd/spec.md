# Spec Delta

## Purpose

The project's automation pipeline: continuous-integration checks that verify every change on Linux, macOS, and Windows, and a tag-triggered release pipeline that builds and publishes downloadable per-platform binaries.

## ADDED Requirements

### Requirement: CI checks run on all supported platforms
On every push to `main` and every pull request update, the pipeline SHALL run formatting check, lint with warnings denied, and tests, on Linux, macOS, and Windows runners.

#### Scenario: Push to main triggers the matrix
- **WHEN** a commit is pushed to `main`
- **THEN** `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` each run on ubuntu, macos, and windows runners

#### Scenario: Pull request gets check results
- **WHEN** a pull request is opened or updated
- **THEN** the same checks run and their pass/fail results are reported on that pull request

#### Scenario: Failing check fails the run
- **WHEN** any check fails on any platform
- **THEN** the workflow run is marked failed for that platform

### Requirement: Version tags publish per-platform release binaries
When a `v*` tag is pushed (or the release workflow is dispatched for a tag), the pipeline SHALL build optimized binaries for Linux x86_64, macOS x86_64, macOS aarch64, and Windows x86_64, and SHALL attach one archive per target to that tag's GitHub release only after every target builds successfully.

#### Scenario: Tag push produces a complete release
- **WHEN** a tag like `v0.1.0` is pushed
- **THEN** the GitHub release for that tag gains four archives — linux-x86_64, macos-x86_64, macos-aarch64 (tar.gz), and windows-x86_64 (zip) — each containing the binary and the README

#### Scenario: Failed target blocks publication
- **WHEN** any target in the release matrix fails to build
- **THEN** the workflow fails and no release or archives are published for that run

### Requirement: Release archives are runnable on stock systems
Each release archive SHALL extract to a binary that starts without installing a Rust toolchain or extra runtime libraries on the target platform.

#### Scenario: Binary runs after download
- **WHEN** a user downloads the archive for their platform from the releases page, extracts it, and runs the binary
- **THEN** the application starts without a Rust toolchain or additional runtime dependencies
