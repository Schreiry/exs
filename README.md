# Exsul

AI business assistant for Georgian SMB — a **Tauri 2 + Rust + Svelte 5** desktop
app built on a clean, recovery-hardened local-first core.

The main screen is a **living context space** (the "void interface"): no pages,
no menus, no chat panel. You type a request and the screen becomes the result —
search, product cards, analysis, AI answer.

> New project. Reuses only the working backend layers of the original
> [Exsul](https://github.com/Schreiry/Exsul) (DB/migrations/inventory/backup/
> command architecture). The old UI was not carried over. App language: Georgian.

## Stack

- **Tauri 2**, Rust 2021
- **Svelte 5** (runes) + SvelteKit (adapter-static) + TypeScript (strict) + Vite
- **SQLite** via `rusqlite` (bundled, WAL), event-sourced projection, **FTS5** search
- Provider-agnostic **AI Gateway** (OpenAI / Gemini / Claude / mock)
- Zod (frontend runtime validation), Serde (Rust)

## Quick start

```sh
npm install
scripts\dev.bat          # or: npm run tauri dev
```

Build an installer:

```sh
scripts\build-release.bat
```

Without API keys the app runs against a **mock** AI provider and seeded demo
products, so the void interface is fully usable offline. Add a key in code via
`ai_set_provider_key` or set `OPENAI_API_KEY` / `GEMINI_API_KEY` /
`ANTHROPIC_API_KEY` in the environment, then select the provider.

## Try it

Type a query into the centre of the screen and press Enter, e.g.
`джип`, `მანქანა`, `красная коробка`, `gift basket`. The scene morphs into
ranked product cards (FTS over name / description / category / AI tags /
multilingual aliases) plus an AI summary.

## Project layout

See [CLAUDE.md](CLAUDE.md) for the full architecture map, branch model, data
model, AI Gateway design, and security rules.

```
src-tauri/   Rust/Tauri core (db, migrations, search, files, ai, commands)
src/         SvelteKit void interface (scene, components, i18n, design tokens)
scripts/     dev.bat / run.bat / build-release.bat
```

## Verify & build (one command)

The strict, fail-fast pipeline lives in two root batch files:

```
check.bat   # verify only — npm check + cargo fmt + cargo check + clippy(-D warnings) + cargo test
build.bat   # everything above, then builds the release installer
```

Any failing stage stops the run with a non-zero exit and names the stage — no
errors slip through. Add `EXSUL_NOPAUSE=1` for CI / non-interactive runs.

Granular equivalents: `npm run check`, `npm run build`,
`cd src-tauri && cargo check && cargo clippy --all-targets -- -D warnings && cargo test`.
