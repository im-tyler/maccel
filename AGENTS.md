# AGENTS.md

This repository is **public** and mirrored to GitHub.
Every commit is publicly visible — treat all work as public-facing.

## Do not commit private/transient context
Never create or commit session, handoff, or local-only context as tracked files:
- No `SESSION_NOTES.md`, `*_NOTES.md`, `HANDOFF*.md`, `*_NEXT_SESSION.md`
- No `.claude/`, `.opencode/` local configs
- No internal completion/tracking docs (`*_PLAN.md`, `*_CHECKLIST.md`, `*PROGRESS*.md`, `*STATUS*.md`) unless intentional public design docs
- No secrets — `.env`, API keys, tokens

These are gitignored as a safety net, but **do not create them in the first place**.

## Commits
- Conventional style (`feat:`, `fix:`, `chore:`, `docs:`).
- History is public — keep it clean.

## Project-specific notes
- Rust workspace, single binary target
- Don't add GUI code to the v0.1 milestone — config is TOML only
- Keep dependencies minimal; this runs as a system daemon
