# ⚡ User Profile Generator

> **Fake identities, real fast.**  
> A Rust TUI tool that spawns random user profiles from [randomuser.me](https://randomuser.me) and lets you copy fields to your clipboard with a single keystroke.

![Rust](https://img.shields.io/badge/rust-%23DEA584.svg?style=flat&logo=rust&logoColor=black)
![TUI](https://img.shields.io/badge/TUI-ratatui-cyan)
![License](https://img.shields.io/badge/license-MIT-blue)

---

https://github.com/user-attachments/assets/9a844c9a-2de7-4192-95a4-15b8709bdc79

---

## 🔥 Features

| What | How |
|------|-----|
| 🎲 **Random profiles** | Fetches fresh users from randomuser.me via `reqwest` |
| ⌨️ **Vim-like nav** | `j`/`k` to move, `Enter`/`Space` to copy, `r` to refresh |
| 📋 **One-click copy** | Any field → clipboard via `arboard` (Wayland & X11) |
| 🔐 **Password rules** | Enforce min length, uppercase, digits, special chars |
| ✉️ **Unique emails** | Auto-appends a random hash before the TLD |

---

## 🚀 Quick Start

```bash
# Nix (recommended)
nix run

# Or build from source
cargo build --release
FIELDS=email,password,first,last ./target/release/user_generator
```

---

## 🎮 Usage

```
┌───────────────────────────────────────────────────────────┐
│                User Profile Generator                      │
├───────────────────────────────────────────────────────────┤
│ > Email        : user@gmailf41a.com                       │
│   Password     : hunter2                                  │
│   First Name   : Eleanor                                  │
│   Last Name    : Nielsen                                  │
│                                                           │
│              New user generated! ✓                        │
├───────────────────────────────────────────────────────────┤
│ ↓/j ↑/k: navigate  Enter/Space: copy  r: refresh  q: quit │
└───────────────────────────────────────────────────────────┘
```

### Controls

| Key | Action |
|-----|--------|
| `↓` / `j` | Move selection down |
| `↑` / `k` | Move selection up |
| `Enter` / `Space` | Copy field to clipboard |
| `r` | Fetch a new random user |
| `q` / `Esc` | Quit |

---

## ⚙️ Configuration

All via environment variables — zero config files.

### Fields

| Variable | Default | Description |
|----------|---------|-------------|
| `FIELDS` | `email,password,first,last` | Comma-separated fields to display |

| Value | Label |
|-------|-------|
| `email` | Email |
| `password` | Password |
| `first` | First Name |
| `last` | Last Name |
| `fullname` | Full Name |
| `username` | Username |

### Password Constraints

| Variable | Default | Description |
|----------|---------|-------------|
| `PASSWORD_MIN_LENGTH` | `8` | Minimum length |
| `PASSWORD_REQUIRE_UPPER` | `false` | Require `A-Z` |
| `PASSWORD_REQUIRE_SPECIAL` | `false` | Require `!@#$%^&*` |
| `PASSWORD_REQUIRE_DIGIT` | `false` | Require `0-9` |

> **Boolean parsing:** `"0"` or `"false"` (case-insensitive) = false. Anything else = true. Unset = default.

### Examples

```bash
# Minimal: just email + password
FIELDS=email,password nix run

# Strict mode: 14-char password with upper + digits
PASSWORD_MIN_LENGTH=14 \
PASSWORD_REQUIRE_UPPER=1 \
PASSWORD_REQUIRE_DIGIT=1 \
nix run

# Hugging Face profile (fields set for a specific platform)
nix run .#huggingface
```

---

## 🏗️ Architecture

| Module | Job |
|--------|-----|
| `api` | HTTP GET → randomuser.me, parse JSON |
| `models` | Serde structs for API response |
| `config` | Env-var config loader |
| `email` | Hash-suffix generator for unique emails |
| `password` | Complexity enforcement engine |
| `app_state` | User data + UI state machine |
| `ui` | ratatui render pass (List, Paragraph, Block) |
| `clipboard_field` | Field enum with label/parse logic |
| `terminal` | Alternate screen / raw mode setup |
| `main` | Event loop — nav, copy, refresh, quit |

---

## 🧪 Development

```bash
nix develop            # Enter dev shell (rustc, cargo, clippy, rustfmt)
cargo build            # Build
cargo test             # Run tests
```

> ⚠️ **Clipboard required.** On Wayland you need `wl-clipboard`; the Nix flake includes it, but `cargo run` outside Nix won't have it.

---

*Made to make form-filling suck less.*
