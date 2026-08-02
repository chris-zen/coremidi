# AGENTS.md

Guidance for AI agents working in this repository.

## Project

Rust library crate wrapping Apple’s CoreMIDI framework. Public API lives in `src/`; low-level FFI comes from `coremidi-sys`. Targets macOS (and historically iOS considerations).

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
cargo deny check
cargo run --example send
```

CI (`.github/workflows/test.yaml`) runs fmt, **cargo-deny** (advisories, bans, licenses, sources), clippy, and tests on `macos-latest` for stable and the MSRV from `rust-version` in `Cargo.toml` (deny runs on `ubuntu-latest`). Tag pushes are ignored by CI; tags trigger deploy instead. Dependabot (`.github/dependabot.yml`) opens weekly PRs for Cargo and GitHub Actions updates.

## Layout

| Path | Role |
|------|------|
| `src/lib.rs` | Crate root, re-exports, `flush` / `restart` |
| `src/client.rs` | Client, notify callbacks, factories |
| `src/object.rs`, `device.rs`, `entity.rs`, `any_object.rs` | Object hierarchy |
| `src/endpoints/` | Sources, destinations, virtual endpoints |
| `src/ports.rs` | Input / output ports |
| `src/packets.rs` | MIDI 1.0 packet lists / buffers |
| `src/events.rs` | MIDI 1.0/2.0 event lists / buffers |
| `src/notifications.rs` | Setup / object / property notifications |
| `src/properties.rs`, `protocol.rs` | Properties and protocol IDs |
| `examples/` | Runnable demos |
| `CHANGELOG.md` | Keep a Changelog; update on every release |

Modules are private; selected types are re-exported from `lib.rs` as the flat public API.

## Architecture notes

- Thin newtypes around CoreMIDI refs (`Object`, `Device`, `Source`, …). Prefer `Deref` / `AsRef` over duplicating object APIs.
- Keep `unsafe` at FFI boundaries (sys calls, CF retain rules, callback pointer casts, packet/event list traversal). Public APIs should stay safe.
- `PacketBuffer` / `PacketList` = legacy MIDI packets; `EventBuffer` / `EventList` = protocol-aware UMP-style events. Prefer protocol-aware APIs for new work.
- `Client` intentionally does **not** dispose on `Drop` (disposing the last client can shut down the MIDI server, especially on iOS). Virtual endpoints and ports *do* dispose on drop; system sources/destinations are non-owning views.
- Errors are usually `Result<_, OSStatus>`; absent lookups return `Option`.
- Unit tests are inline `#[cfg(test)]` modules, not a top-level `tests/` tree.

## Conventions

- Treat `rust-version` in `Cargo.toml` as the MSRV source of truth (do not hardcode it elsewhere).
- Match existing style: doc comments with CoreMIDI Apple doc links, no drive-by refactors, no new comments unless needed for non-obvious invariants.
- Do not commit `Cargo.lock` (gitignored for this library), `.idea/`, `.vscode/`, or local scratch such as `src/NOTES.md`.
- Do not add features that require non-Apple targets unless explicitly requested.

## Releases

1. Bump `version` in `Cargo.toml`.
2. Update `CHANGELOG.md` (Keep a Changelog sections; add compare link at bottom).
3. Commit on `master` (message like `Bump version to X.Y.Z`).
4. Tag with the bare SemVer string (**no `v` prefix**), e.g. `0.9.2`.
5. Push commit and tag. Tag push runs `.github/workflows/deploy.yaml` (docs → GitHub Pages, then `cargo publish`).

Ensure the git tag matches `Cargo.toml` before pushing.

## Dependencies

- `coremidi-sys` — FFI bindings; bump carefully and run full `cargo test` on macOS.
- `core-foundation` / `core-foundation-sys` — CF string/property interop.
- `block2` — Objective-C blocks for CoreMIDI callbacks.

## Supply chain

- Policy lives in `deny.toml`. Enforce with `cargo deny check` (advisories, licenses, sources, bans).
- Dependencies must come from crates.io only; no git/path deps in published graphs.
- Allowed licenses are listed in `deny.toml`; widen the allowlist only with intent.
- Duplicate crate versions and version wildcards are denied.
- When Dependabot or a manual bump fails deny checks, fix the graph or add a dated, reasoned exception in `deny.toml` — do not disable the CI job.
