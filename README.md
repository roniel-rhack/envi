# envi

A terminal UI for managing `.env` files — diff, scan, edit, and validate environment variables across projects and profiles.

<p align="center">
  <img src="demo.gif" alt="envi demo" width="800" />
</p>

## Why?

Every project has `.env` files. Managing them is painful:

- You pull a repo and your `.env` is missing vars that were added to `.env.example`
- You can't remember which env vars go where across 10 projects
- Comparing dev vs production configs means opening two files side by side
- You have no idea which env vars in your `.env` are actually used in code
- Sensitive values sit in plaintext

**envi** solves all of this in a fast, keyboard-driven TUI.

## Install

### Homebrew (macOS / Linux)

```bash
brew tap roniel-rhack/tap
brew install envi
```

### Download binary

Grab the latest binary for your platform from the [Releases](https://github.com/roniel-rhack/envi/releases) page.

Available for: **macOS** (arm64, amd64), **Linux** (arm64, amd64), **Windows** (amd64).

### From source (requires Rust)

```bash
git clone https://github.com/roniel-rhack/envi
cd envi
cargo build --release
# Binary at ./target/release/envi
```

## Usage

```bash
# Run in current directory
envi

# Run in a specific directory
envi /path/to/project
```

## Features

### Browse & Edit

Navigate through your `.env` files with vim-style keybindings. Edit values inline. Add and delete variables.

### Diff View (`d`)

Compare any two `.env` files side by side. Instantly see:
- **Missing** vars (in source but not target)
- **Extra** vars (in target but not source)
- **Changed** values between profiles
- Press `Tab` to cycle through diff targets

### Code Scanner (`s`)

Scans your entire project source code for environment variable references. Detects patterns across 10+ languages:
- `process.env.VAR` (JS/TS)
- `os.environ["VAR"]` / `os.getenv("VAR")` (Python)
- `env::var("VAR")` (Rust)
- `os.Getenv("VAR")` (Go)
- `System.getenv("VAR")` (Java)
- `${VAR}` / `$VAR` (Shell/Docker/YAML)
- And more...

Reports:
- Vars used in code but **not defined** in any `.env` file
- Vars defined in `.env` but **never used** in code

### Search (`/`)

Fuzzy search across variable names and values with live matching.

### Validation

Automatic warnings for:
- Empty values
- Potentially sensitive unencrypted values
- Non-UPPER_SNAKE_CASE keys
- Variables missing from other profiles

### Profile Awareness

Auto-discovers all `.env` variants: `.env`, `.env.local`, `.env.development`, `.env.staging`, `.env.production`, `.env.example`, `.env.test`, and any other `.env.*` files.

## Keybindings

| Key | Action |
|-----|--------|
| `j`/`k` or `↓`/`↑` | Navigate up/down |
| `h`/`l` or `←`/`→` | Switch profile |
| `Tab` / `Shift+Tab` | Switch panel |
| `e` or `Enter` | Edit selected value |
| `a` | Add new variable |
| `x` | Delete variable |
| `w` | Save file |
| `r` | Reload all files |
| `d` | Toggle diff view |
| `s` | Toggle code scan |
| `/` | Search |
| `n` | Next search match |
| `?` | Help |
| `q` / `Esc` | Quit |

## Tech

- **Rust** — fast, safe, single binary
- **ratatui** — modern TUI framework
- **crossterm** — cross-platform terminal handling
- **2 MB** binary, zero runtime dependencies

## License

MIT
