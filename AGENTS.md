# AGENTS.md — Exsul (new AI business assistant core)

Guidance for working in this repository. Read this before editing.

## What this is

Exsul is a **Tauri 2 + Rust + Svelte 5 + TypeScript** desktop app for Georgian
SMB owners. The main screen is a **"void interface"** — a single living context
space (not pages/menus/chat panels). The user types a request and the scene
morphs into search results, product cards, analysis, etc.

This is a **new project** that reuses only the working backend layers of the old
Exsul (`github.com/Schreiry/Exsul`). The old UI/domain was intentionally NOT
ported. App language is **Georgian**; code comments are mixed RU/EN.

## Branch model

| Branch | Owns |
|---|---|
| `core` (current) | Rust/Tauri kernel: DB, migrations, file system, import/export, backup, typed commands, FTS search |
| `business` | Entrepreneurial scenarios, API integrations, analytics, competitors, gov/stat services |
| `interface` | New UI, void space, animations, design system, Svelte components |
| `ai` | AI Gateway providers, prompt templates, vision, embeddings, Georgian localization logic |
| `dev` | Integration branch (core/business/interface/ai merge here after review) |

**Layer rules:** Core must not depend on UI. UI must not touch SQLite directly —
always go through typed Tauri commands. AI must not mutate the DB except through
the command/service layer. Business modules call core via stable interfaces.

> This branch currently also contains a *thin but real* AI Gateway and a minimal
> void interface so the concept is buildable end-to-end. When the team splits
> work, `src-tauri/src/ai/**` belongs to `ai` and `src/**` to `interface`.

## Architecture map

```
src-tauri/src/
  lib.rs              # Tauri wiring: panic hook → preflight → recovery DB → HLC → commands → backup-on-close
  main.rs             # entry
  preflight.rs        # pre-Tauri writable-dir check (clear error instead of flash-and-die)
  db/
    mod.rs            # aggressive auto-heal DB init (NEVER panics) — ported from Exsul
    migrations.rs     # robust migration runner (smart_split, idempotent retry) — ported
    queries.rs        # items / categories / audit / events / AI metadata / analytics
    seed.rs           # demo Georgian products (idempotent)
  events/
    types.rs          # Item, payloads, AiItemMetadata, InventorySummary (serde contract)
    store.rs          # append_event / append_audit_log — ported
  sync/hlc.rs         # Hybrid Logical Clock (event timestamps) — ported
  search/mod.rs       # FTS5 search + scoring → ProductSearchResponse; reindex_item_fts
  files/
    mod.rs            # product image storage
    backup.rs         # zip backup/restore via rusqlite online-backup API
  ai/                 # provider-agnostic AI Gateway
    types.rs provider.rs router.rs            # contract + trait + fallback router
    openai.rs gemini.rs Codex.rs mock.rs     # providers (mock = no-key dev/tests)
    prompts.rs vision.rs localization.rs      # templates, JSON parse, KA second-pass
    secrets.rs                                # keyring-based API-key storage
  commands/           # typed Tauri command API (inventory, search, ai, categories, audit, backup, system)
  migrations/         # 001 core schema, 002 projection triggers, 003 ai_metadata, 004 FTS5

src/                  # SvelteKit "void interface"
  app.css             # design tokens (graphite void / quiet luxury)
  lib/types.ts        # TS mirror of Rust structs
  lib/schemas.ts      # Zod runtime validation at the command boundary
  lib/tauri/commands.ts  # typed command wrappers (only place that calls invoke)
  lib/i18n/           # Georgian dictionary + t() helper
  lib/scene/scene.svelte.ts  # scene state machine (runes) + browser mock fallback
  lib/components/     # VoidBackground, ContextInput, ProductCard, ResultScene
  routes/+page.svelte # main screen orchestration + pointer parallax
```

### Data model (event-sourced)

`events` is an append-only ledger; SQL triggers (migration 002) project item
events (`ItemCreated`/`ItemUpdated`/`StockAdjusted`/`PriceChanged`/`SaleRecorded`)
into the `items` table. Commands append events; they never mutate `items`
directly (except UI-only `card_color`). FTS index is maintained from Rust
(`search::reindex_item_fts`) after each write — more robust than multi-table
triggers.

## Commands to run

**Canonical (fail-fast, strict):**
```
build.bat   # FULL: npm check -> cargo fmt --check -> cargo check -> clippy -D warnings -> cargo test -> tauri build (release)
check.bat   # VERIFY only (no release bundle) — same strict checks, fast; run this often during dev
```
Both stop on the first failing stage with a non-zero exit and print which stage
failed — nothing gets through. Set `EXSUL_NOPAUSE=1` to skip the final `pause`
(CI / non-interactive). `scripts\build-release.bat` delegates to `build.bat`.

**Granular:**
```
scripts\dev.bat            # Tauri + Vite dev (hot reload)
scripts\run.bat            # launch built binary
npm run check              # svelte-check + tsc (strict)
npm run build              # build frontend to build/
cd src-tauri && cargo check    # type-check Rust
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo test     # unit tests (migrations, queries, search, prompts)
```

`generate_context!` requires `build/` to exist — run `npm run build` before
`cargo check` on a clean checkout (or keep the placeholder).

## AI Gateway

Provider-agnostic. `ai::build_router(selected)` builds an ordered provider list
(selected first, then other configured providers as fallbacks; `mock` if none
configured). Add a provider by implementing `ai::provider::AiProvider`.

- **OpenAI** uses Chat Completions (stable, vision + JSON mode). The spec calls
  for the Responses API — that swap is isolated to `openai.rs` (next step).
- **Codex** uses Anthropic Messages API, model `Codex-opus-4-8` (no
  `temperature`/`thinking` params on Opus 4.8).
- **Gemini** uses the Generative Language API.
- API keys live in OS keyring (`ai/secrets.rs`), with env-var fallback for dev
  (`OPENAI_API_KEY` / `GEMINI_API_KEY` / `ANTHROPIC_API_KEY`).

## Security rules (enforce these)

1. Never commit or hardcode API keys. Use `ai::secrets` (keyring) or env.
2. Never send the whole DB to a provider — only minimal relevant context
   (`assistant_query` sends top-N search hits only).
3. For image analysis, send only the selected image.
4. Log AI errors **without** secrets (providers log HTTP status only).
5. Network calls have timeouts (`ai::http_client`, 45s).
6. Do not write private business data or secrets to `audit_logs`/debug logs.

## Conventions

- TypeScript strict; validate command results with Zod at the boundary.
- Rust: `Result<_, String>` for command errors; `Arc<Mutex<Connection>>` for DB;
  never hold a `MutexGuard` or `State<'_, _>` across `.await` (clone the `Arc`).
- Georgian text must be natural (meaning-based), not literal translation.
- i18n keys are nested + descriptive; add strings to `src/lib/i18n/ka.ts`.
- Migrations are additive and sequentially numbered; don't rewrite old ones.
