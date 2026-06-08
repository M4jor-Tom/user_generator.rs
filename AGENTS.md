# AGENTS.md

## Project

Rust CLI tool (`user_generator`) — fetches random users from `randomuser.me` API, displays them in a `ratatui` TUI, and copies fields to the clipboard via `arboard`.

## Commands

```
nix develop          # Enter dev shell (rustc, cargo, clippy, rustfmt, rust-analyzer)
nix run              # Run default profile
nix run .#huggingface # Run Hugging Face profile (stricter password rules)
cargo build          # Build
cargo test           # Run tests (none exist yet)
```

CI (`.github/workflows/rust.yml`) runs `cargo build --verbose` then `cargo test --verbose` — no clippy or rustfmt checks.

## Nix gotcha

The flake uses `cargo vendor` (via a fixed-output derivation) instead of nix's built-in
Python crate fetcher, because crates.io blocks the Python fetcher's User-Agent on some
networks. When `Cargo.toml`/`Cargo.lock` changes, update the `outputHash` of
`cargoVendorDir` in `flake.nix`. To get the new hash:

    nix build --no-write-lock-file 2>&1 | grep -oP 'got:\s*\K.*' | head -1

## Runtime requirements

- `wl-clipboard` (Wayland) or X11 clipboard — the binary calls `arboard::Clipboard` which requires a running clipboard daemon.
- The nix flake wrappers include `wl-clipboard`; raw `cargo run` will fail without it.

## Configuration

All config via environment variables (no config files):

| Variable | Default | Purpose |
|---|---|---|
| `FIELDS` | `email,password,first,last` | Comma-separated fields to display/copy |
| `PASSWORD_MIN_LENGTH` | `8` | Min password length |
| `PASSWORD_REQUIRE_UPPER` | `false` | Enforce uppercase in password |
| `PASSWORD_REQUIRE_SPECIAL` | `false` | Enforce special char in password |
| `PASSWORD_REQUIRE_DIGIT` | `false` | Enforce digit in password |

Boolean parsing: `"0"` or `"false"` (case-insensitive) = false; any other non-empty value = true; unset = default.

Supported field names: `email`, `password`, `first`, `last`, `fullname`, `username`. Unknown fields warn to stderr and are skipped; zero valid fields = panic.

## Architecture

Flat module structure under `src/` — single binary, no library crate. Entry point is `src/main.rs:22`. Modules are self-contained utility functions and structs (no inter-module dependencies beyond what `main.rs` wires together).
