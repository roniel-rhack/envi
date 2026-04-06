<div align="center">

# envi

**Stop editing `.env` files blind.**

Diff, scan, edit, and validate environment variables across all your profiles — right from the terminal.

[![CI](https://github.com/roniel-rhack/envi/actions/workflows/ci.yml/badge.svg)](https://github.com/roniel-rhack/envi/actions/workflows/ci.yml)
[![Release](https://github.com/roniel-rhack/envi/releases/latest/badge.svg)](https://github.com/roniel-rhack/envi/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<img src="assets/demo.gif" alt="envi demo" width="800" />

</div>

---

## The Problem

Every developer knows the pain:

- You pull a repo and `.env` is missing 5 vars that were added to `.env.example`
- Comparing dev vs production means opening two files and squinting
- You have no idea which env vars your code actually uses
- Sensitive values sit in plaintext, one `git add .` away from disaster

**envi** gives you a single dashboard for all of it.

## Install

### Homebrew (macOS / Linux)

```bash
brew tap roniel-rhack/tap
brew install envi
```

### Download binary

Grab the latest from [**Releases**](https://github.com/roniel-rhack/envi/releases) — available for **macOS** (arm64, amd64), **Linux** (arm64, amd64), and **Windows** (amd64).

### From source

```bash
git clone https://github.com/roniel-rhack/envi && cd envi
cargo build --release
# Binary at ./target/release/envi (~2 MB)
```

## Quick Start

```bash
# Run in current directory
envi

# Run in a specific project
envi ~/projects/my-app
```

envi auto-discovers all `.env` variants: `.env`, `.env.local`, `.env.development`, `.env.staging`, `.env.production`, `.env.example`, `.env.test`, and any `.env.*` file.

## Features

### Browse & Edit

Navigate your env files with vim-style keys. Edit values inline. Add or delete variables. Save when you're ready.

### Diff View  `d`

Compare any two `.env` files instantly:

| Symbol | Meaning |
|--------|---------|
| `- MISSING` | In source but not in target |
| `+ EXTRA` | In target but not in source |
| `~ CHANGED` | Different values between files |

Press `Tab` to cycle through diff targets.

### Code Scanner  `s`

Scans your project for env var references across **10+ languages**:

```
process.env.VAR        (JS/TS)          os.environ["VAR"]    (Python)
env::var("VAR")        (Rust)           os.Getenv("VAR")     (Go)
System.getenv("VAR")   (Java)           ENV["VAR"]           (Ruby)
${VAR} / $VAR          (Shell/Docker)   getenv("VAR")        (PHP/C)
```

Reports vars **used in code but not defined**, and vars **defined but never used**.

### Search  `/`

Live fuzzy search across variable names and values.

### Validation

Automatic warnings for:
- Empty values
- Potentially sensitive unencrypted values
- Non-UPPER_SNAKE_CASE keys
- Variables missing from other profiles

### Cross-File Awareness

The details panel shows which other profiles contain (or are missing) the selected variable — no more guessing.

## Keybindings

| Key | Action |
|:---:|--------|
| `j` `k` | Navigate up / down |
| `h` `l` | Previous / next profile |
| `Tab` | Switch panel |
| `e` | Edit value |
| `a` | Add variable |
| `x` | Delete variable |
| `w` | Save |
| `r` | Reload |
| `d` | Diff view |
| `s` | Code scan |
| `/` | Search |
| `n` | Next match |
| `?` | Help |
| `q` | Quit |

## Built With

| | |
|---|---|
| **Language** | Rust — fast, safe, single binary |
| **TUI** | [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) |
| **Binary size** | ~2 MB |
| **Dependencies** | Zero runtime dependencies |
| **Platforms** | macOS, Linux, Windows |

## License

[MIT](LICENSE)
