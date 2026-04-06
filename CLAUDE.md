# envi — CLAUDE.md

## Project Overview

**envi** is a terminal UI (TUI) for managing `.env` files — diff, scan, edit, and validate environment variables across projects and profiles. Built in Rust with ratatui + crossterm.

## Build & Test

```bash
cargo build              # Dev build
cargo build --release    # Release build (LTO + strip, ~2MB)
cargo test               # Run all tests
cargo fmt --all          # Format code
cargo clippy -- -D warnings  # Lint (must pass clean)
```

## Project Structure

```
src/
├── main.rs            # CLI entry, terminal setup, event loop, key handling
├── app.rs             # Central app state, all mutations, navigation, editing
├── env/
│   ├── parser.rs      # .env file parser (quotes, comments, export, encryption detection)
│   ├── diff.rs        # Diff engine — compare two EnvFiles, report missing/extra/changed
│   └── scanner.rs     # Code scanner — regex patterns for 10+ languages to find env var usage
└── ui/
    ├── layout.rs      # Main 3-panel layout + status bar rendering
    ├── profiles.rs    # Left panel — file/profile list
    ├── variables.rs   # Center panel — variable list, diff view, scan view
    ├── details.rs     # Right panel — variable details, cross-file presence, warnings
    ├── dialogs.rs     # Modal overlays — help, search, confirm
    └── helpers.rs     # Shared UI utilities — safe string truncation, char chunking
```

## Conventions

- **Rust 2021 edition**
- All clippy warnings are errors in CI (`-D warnings`)
- `cargo fmt` enforced — no style debates
- Tests live in `#[cfg(test)] mod tests` inside each module
- Vim-style keybindings (j/k/h/l) as primary, arrow keys as secondary
- All app state mutations go through `App` methods in `app.rs` — UI is purely rendering

## CI/CD

- **CI** (`ci.yml`): check, test, fmt, clippy on push/PR to main
- **Release** (`release.yml`): triggered by `v*` tags, builds 5 platform binaries, creates GitHub release, triggers homebrew tap update
- **Homebrew tap**: `roniel-rhack/homebrew-tap` — auto-updated via repository_dispatch

## Release Checklist

**REQUIRED** every time a new version is tagged and released:

1. Bump version in `Cargo.toml`
2. Commit, tag (`v*`), and push: `git push origin main --tags`
3. Wait for the release workflow to complete
4. **Update release notes** via `gh release edit <tag> --notes "..."` with:
   - Version header and one-line summary
   - Categorized changes: Bug Fixes, Features, Performance, Code Quality, Breaking Changes (if any)
   - Install instructions (Homebrew + binary download)
   - Link to full changelog
5. Verify the release page looks correct on GitHub

## Workflow Orchestration

### 1. Plan Mode Default
- Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
- If something goes sideways, STOP and re-plan immediately — don't keep pushing
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity

### 2. Subagent Strategy
- Use subagents liberally to keep main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One task per subagent for focused execution

### 3. Self-Improvement Loop
- After ANY correction from the user: update `tasks/lessons.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project

### 4. Verification Before Done
- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Run tests, check logs, demonstrate correctness

### 5. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes — don't over-engineer
- Challenge your own work before presenting it

### 6. Autonomous Bug Fixing
- When given a bug report: just fix it. Don't ask for hand-holding
- Point at logs, errors, failing tests — then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

## Task Management

1. **Plan First**: Write plan to `tasks/todo.md` with checkable items
2. **Verify Plan**: Check in before starting implementation
3. **Track Progress**: Mark items complete as you go
4. **Explain Changes**: High-level summary at each step
5. **Document Results**: Add review section to `tasks/todo.md`
6. **Capture Lessons**: Update `tasks/lessons.md` after corrections

## Core Principles

- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No Laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimal Impact**: Changes should only touch what's necessary. Avoid introducing bugs.
