# User Profile Generator

A Rust CLI tool that fetches random user profiles from the [randomuser.me API](https://randomuser.me/) and copies individual fields to the clipboard one-by-one, enabling quick copy-paste workflows for filling forms or creating test accounts.

## Features

### Random User Fetching

Performs a synchronous HTTP GET request to `https://randomuser.me/api/` using `reqwest` with rustls TLS. Extracts four data points from the first result: first name, last name, email, and password. Each iteration fetches a fresh, independent user.

### Interactive Clipboard Copying Workflow

After fetching and displaying a user profile, the tool enters a step-by-step clipboard flow:

1. Waits for you to press **Enter** (gives time to focus the target input field)
2. Copies the first configured field to the clipboard and confirms the value
3. Waits for **Enter** before copying the next field
4. Repeats until all fields are copied

This lets you switch to the target application, paste, switch back, then press Enter for the next field -- all without manually selecting text or opening a clipboard manager.

### Email Hash Suffix

Each generated email automatically receives a 4-character lowercase hexadecimal suffix appended before the TLD, ensuring uniqueness across sessions.

```
john@gmail.com  →  john@gmailf41a.com
```

The hash is generated per-user using `rand` with `gen_range(0..=15)`, producing values like `a3c7`, `f01e`, etc. If the email cannot be parsed (no `@` or no `.` in domain), the original email is returned unmodified.

### Password Adjustment

The raw password from the API is optionally adjusted to meet complexity rules defined via environment variables:

- **Uppercase enforcement** -- inserts a random `A`-`Z` if missing
- **Special character enforcement** -- inserts a random character from `!@#$%^&*()_+-=[]{}|;:,.<>?` if missing
- **Digit enforcement** -- inserts a random `0`-`9` if missing
- **Minimum length padding** -- pads with random characters from `[a-z0-9!@#$%^&*]` if below threshold

If no restrictions are configured and the original password already meets the minimum length, it is returned unchanged.

### Profile Display

Renders a bordered, auto-sizing ASCII table with all four fields aligned to the longest value.

### Regeneration Loop

After completing a clipboard copy cycle, you are prompted to generate another user. Type `y` to continue; any other input exits.

## Installation

### Prerequisites

- Rust toolchain (1.70+)
- A clipboard-compatible environment (X11, Wayland, macOS, or Windows)

### Build

```bash
cargo build --release
```

The binary is at `target/release/user_generator`.

### Nix

```bash
nix develop    # enter dev shell with Rust toolchain
nix build      # build the package
```

## Usage

```bash
user_generator
```

The tool has no CLI arguments. All configuration is done via environment variables.

### Examples

```bash
# Basic usage with defaults
./target/release/user_generator

# Only copy email and password (no name fields)
FIELDS=email,password ./target/release/user_generator

# Require uppercase and digits in passwords
PASSWORD_REQUIRE_UPPER=1 PASSWORD_REQUIRE_DIGIT=1 ./target/release/user_generator

# Full example with all options
FIELDS=email,password,first \
PASSWORD_MIN_LENGTH=12 \
PASSWORD_REQUIRE_UPPER=1 \
PASSWORD_REQUIRE_SPECIAL=1 \
PASSWORD_REQUIRE_DIGIT=1 \
./target/release/user_generator
```

## Configuration

Configuration is loaded from environment variables at startup.

### Field Selection

| Variable | Default | Description |
|----------|---------|-------------|
| `FIELDS` | `email,password,first,last` | Comma-separated list of fields to copy, in order |

Recognized field names (case-insensitive, whitespace-trimmed):

| Name | Label |
|------|-------|
| `email` | Email |
| `password` | Password |
| `first` | First Name |
| `last` | Last Name |

Unknown fields trigger a warning to stderr and are skipped. If no valid fields remain, the program panics.

### Password Constraints

| Variable | Default | Description |
|----------|---------|-------------|
| `PASSWORD_MIN_LENGTH` | `8` | Minimum required password length |
| `PASSWORD_REQUIRE_UPPER` | `false` | Force at least one ASCII uppercase letter |
| `PASSWORD_REQUIRE_SPECIAL` | `false` | Force at least one special character |
| `PASSWORD_REQUIRE_DIGIT` | `false` | Force at least one ASCII digit |

#### Boolean Parsing

Boolean env vars are `false` only if the value equals `"0"` or `"false"` (case-insensitive). Any other non-empty value is `true`. Unset variables fall back to the hardcoded default.

## Session Flow

```
$ ./target/release/user_generator

User Profile Generator
Fields: ["Email", "Password", "First Name", "Last Name"]
Password restrictions: min_length=8

Fetching user from randomuser.me...

╔══════════════════════════════════════════╗
║          Generated User Profile          ║
╠══════════════════════════════════════════╣
║  First Name : Eleanor                    ║
║  Last Name  : Nielsen                    ║
║  Email      : eleanor.nielsen@gmailc4a1.com ║
║  Password   : hunter2                    ║
╚══════════════════════════════════════════╝

Select the terminal, then press Enter to begin clipboard insertion.
[Enter pressed]

  Copying Email...
    ✓ Copied: eleanor.nielsen@gmailc4a1.com
  Press Enter to copy the next field...[Enter]
  Copying Password...
    ✓ Copied: hunter2
  Press Enter to copy the next field...[Enter]
  Copying First Name...
    ✓ Copied: Eleanor
  Press Enter to copy the next field...[Enter]
  Copying Last Name...
    ✓ Copied: Nielsen

All fields copied to clipboard!
Generate another user? (Y/n):
```

### Code Organization (inside `main.rs`)

1. **API Response Models** -- `serde::Deserialize` structs for `randomuser.me` JSON
2. **Clipboard Field Enum** -- `ClipboardField` with `from_str` and `label` methods
3. **Configuration** -- `Config` struct loaded from environment variables
4. **Utility Functions** -- `modify_email()`, `adjust_password()`
5. **Core Functions** -- `fetch_user()`, `display_profile()`
6. **Main Loop** -- interactive fetch-display-copy-regenerate cycle
