# AI Execution Plan — Exsul (ai branch)

Consolidated, ordered action list for the next AI agent (or human) working on
the `ai` branch. This document captures the verified state of the codebase, the
specific bugs and wiring gaps identified in prior review sessions, and the
exact remediation steps required.

**Branch context:** `ai` is one commit ahead of `origin/AI` after pulling from
`origin/Core`. Core-side AI Gateway code lives in `src-tauri/src/ai/`. Frontend
mirror lives in `src/lib/tauri/commands.ts` and `src/lib/types.ts`.

---

## 0. Verified foundational state (do not break)

These were cross-checked against the actual code on disk and confirmed correct.
Any change in this section needs a written justification, not a silent rewrite.

### 0.1 Provider abstraction
- `ai::provider::AiProvider` trait (`src-tauri/src/ai/provider.rs`):
  `kind()`, `is_configured()`, `answer()`, `analyze_image()`.
- `ai::router::AiRouter` (`router.rs`): tries providers in order, returns the
  last error if all fail. No panics on full failure.
- `ai::build_router(selected)` (`mod.rs:52`): selected first, then other
  configured real providers as fallbacks, then `MockProvider` if nothing real
  is configured. Fallback order is fixed:
  `REAL_PROVIDERS = [OpenAi, Gemini, Claude]`.

### 0.2 Security / context hygiene
- HTTP client (`ai::http_client` in `mod.rs:29`) enforces `timeout = 45s`,
  `connect_timeout = 10s`.
- `commands::ai::assistant_query` (`commands/ai.rs:40`) caps grounding context
  to `MAX_CONTEXT_ITEMS = 8` items, mapped to a narrow `ProductContext`
  (`item_id`, `name`, `description`, `category`, `price`, `tags`). The whole
  DB is **never** sent.
- API keys: `ai::secrets` uses OS keyring (`com.exsul.app` service) with env-var
  fallback (`OPENAI_API_KEY`, `GEMINI_API_KEY`, `ANTHROPIC_API_KEY`). Keys
  never appear in logs or audit logs.
- `AiError` enum (`ai/types.rs:100`) — `Display` impl never includes the raw
  provider body or key material. Only HTTP status code is logged on failure.

### 0.3 Vision JSON handling
- `prompts::extract_json_object` (`ai/prompts.rs:84`) is a real depth-counting
  extractor: handles `\` escapes inside strings, ignores braces inside string
  literals, returns the first balanced top-level object. Has a unit test
  (`prompts.rs:121`) that exercises prose + ` ```json ` fences + escaped quotes.
- `vision::parse_vision_json` deserializes into `VisionMetadata`
  (`ai/types.rs:78`): `caption_ka/ru/en`, `tags`, `visual_attributes`,
  `aliases`, `confidence`.

### 0.4 Georgian prompt contract
- `prompts::system_answer(language)` (`ai/prompts.rs:8`) switches system
  instructions per language. The `ka` branch explicitly forbids literal
  translation and Russian calques, demands natural business phrasing.
- `prompts::answer_user_message(req)` (`ai/prompts.rs:46`) renders
  `ProductContext` list as a `Relevant products:` block followed by the user
  query.

### 0.5 DB & app infrastructure (not in scope of this plan, but verified)
- `preflight::run_preflight` (`src-tauri/src/preflight.rs:24`) writes a probe
  file before Tauri init to catch no-write-permission failures cleanly.
- `db::init_with_recovery` (`src-tauri/src/db/mod.rs:192`) cascade:
  attempt 1 open → attempt 2 quarantine+recover → attempt 3 catastrophic
  (nuke + in-memory fallback). Never panics.
- `db::migrations` has idempotent `run()` (`src-tauri/src/db/migrations.rs:246`)
  with `apply_migration_safe` → `apply_statement_by_statement` → `smart_split`.
- Backup/restore uses rusqlite's online backup API to a zip
  (`src-tauri/src/files/backup.rs`).
- Event-sourced projection: append `ItemCreated`/`ItemUpdated`/
  `StockAdjusted`/`PriceChanged`/`SaleRecorded` to `events`; SQL triggers
  (migration 002) project into `items`. FTS reindex is done from Rust
  (`search::reindex_item_fts`) after each write — more robust than triggers.

---

## 1. Bugs to fix (in priority order)

Each item lists: file, line(s), the exact problem, the required fix, and the
verification step. Apply in this order so the build stays green between
commits.

### 1.1 [HIGH] `claude.rs` — wrong model tier for this use case
- **File:** `src-tauri/src/ai/claude.rs`
- **Lines:** 4 (comment), 14 (`DEFAULT_MODEL`)
- **Problem:** Model is `claude-opus-4-8` — exists, but is the most expensive
  tier and overkill for short answers + vision tagging. Also, the comment on
  line 4 (`на Opus 4.8 НЕ передаём temperature/thinking — это 400`) is no
  longer applicable once the model is changed.
- **Fix:**
  1. Set `DEFAULT_MODEL` to `"claude-haiku-4-5-20251001"` (fast, cheap,
     vision-capable, current generation).
  2. Delete the stale comment on line 4.
  3. Leave the body construction unchanged — Haiku accepts `temperature` /
     `thinking` if you ever want to pass them later, but the current
     `max_tokens`-only body is fine.
- **Verify:** `cargo check` clean; live test with `ANTHROPIC_API_KEY` env var
  and `analyze_item_image` produces valid JSON via `vision::parse_vision_json`.

### 1.2 [HIGH] `gemini.rs` — dead model string
- **File:** `src-tauri/src/ai/gemini.rs`
- **Line:** 11 (`DEFAULT_MODEL`)
- **Problem:** Model is `gemini-1.5-flash`. Gemini 1.5 was shut down; calls
  return HTTP 404. Provider is effectively unreachable.
- **Fix:** Set `DEFAULT_MODEL` to `"gemini-flash-latest"` — auto-updating alias
  that resolves to the current Gemini Flash (currently Gemini 3.5 Flash).
- **Verify:** Live call with `GEMINI_API_KEY` env var to `analyze_image` and
  `answer` returns 200 + parseable body.

### 1.3 [HIGH] `gemini.rs` — missing structured-output flag for vision
- **File:** `src-tauri/src/ai/gemini.rs`
- **Lines:** 78–91 (`analyze_image`)
- **Problem:** `analyze_image` only passes prompt instructions and an inline
  image. No `generationConfig.response_mime_type: "application/json"` — model
  is free to wrap output in prose, which `extract_json_object` then has to
  strip. Inconsistent with OpenAI, which sets `response_format: json_object`.
- **Fix:** In the request body, wrap `contents` inside an outer object that
  also includes `generationConfig`:
  ```rust
  json!({
      "contents": contents,
      "generationConfig": { "responseMimeType": "application/json" }
  })
  ```
  Pass that to `self.generate(...)` (adjust signature if needed — currently
  `generate` takes `contents` directly; add an optional `generation_config`
  param or take a full body).
- **Verify:** Live `analyze_image` call returns pure JSON (no ` ```json `
  fence). `extract_json_object` still passes its unit test.

### 1.4 [MEDIUM] `openai.rs` — stale model tier
- **File:** `src-tauri/src/ai/openai.rs`
- **Line:** 13 (`DEFAULT_MODEL`)
- **Problem:** `gpt-4o-mini` is a deprecated/stale tier. Aliases shift, so the
  exact replacement must be confirmed at implementation time (check
  `https://platform.openai.com/docs/models` on the day of the change). Note:
  this spec was written before the rename to GPT-5.x; the model string to use
  must be verified against current OpenAI docs.
- **Fix:** Set `DEFAULT_MODEL` to a current affordable GPT-5.x-tier model
  alias. Record the chosen string and the date it was verified in the commit
  message.
- **Verify:** Live call to `answer` and `analyze_image` returns 200 + valid
  body. Update the comment in `openai.rs:1–4` if the API surface changes.

### 1.5 [LOW] `claude.rs` — no structured-output flag for vision
- **File:** `src-tauri/src/ai/claude.rs`
- **Lines:** 88–106 (`analyze_image`)
- **Problem:** Like Gemini, relies on prompt instructions alone. Claude has a
  real structured-output mechanism but it's not trivially mapped to the same
  `response_format` shape as OpenAI/Gemini.
- **Decision needed:** Either (a) leave as-is — `extract_json_object` already
  handles prose-wrapped JSON, and Claude tends to obey strict-JSON prompts,
  or (b) use Claude's `tools`/`output_format` mechanism (current Anthropic
  API supports `output_config.format` for JSON schemas).
- **Recommended:** Defer until 1.1/1.2/1.3 ship and Claude is observed to leak
  prose in practice. If fixing: add `output_config` to the body for `claude`
  model and document in this file.

---

## 2. Wiring gap — Georgian second-pass localization

### 2.1 [HIGH] `localization::georgian_review` is unreachable from the UI
- **File:** `src-tauri/src/ai/localization.rs`
- **Lines:** 11–25
- **Problem:** Fully implemented, marked `#[allow(dead_code)]`, never invoked.
  Spec requirement #13 (Georgian localization quality check) is in the code
  but not in the user-visible flow.
- **Two valid fixes** — pick one and document the choice in the commit:

  **Option A — Auto-invoke inside `analyze_item_image`**
  - In `commands::ai::analyze_item_image` (`commands/ai.rs:107`), after
    `router.analyze_image(...)` returns `meta`, take the first configured
    provider via `router.first()` and pass each non-empty caption (`ka`,
    `ru`, `en`) through `ai::localization::georgian_review(...)` before
    building `AiItemMetadata`.
  - Caveat: only run the KA pass over `caption_ka`; the `ru` and `en`
    captions can go through the same provider's `answer` flow with a
    English/Russian review instruction if desired, but that's out of scope.
  - Drop the `#[allow(dead_code)]` on `georgian_review` and on
    `prompts::georgian_review_instruction`.
  - Remove `#[allow(dead_code)]` on `router::AiRouter::first()` (line 54).
  - Caveat: this doubles vision-call cost. Acceptable for catalog-on-import
    use case; flag in CLAUDE.md.

  **Option B — New standalone Tauri command**
  - Add `#[tauri::command] pub async fn ai_review_georgian(
        db: State<'_, Database>,
        text: String) -> Result<String, String>` in `commands/ai.rs`.
  - Reads selected provider, builds router, runs `localization::georgian_review`
    over the first provider.
  - Frontend: add a "review" affordance next to captions in the product card
    editor, calling this command. Expose wrapper in
    `src/lib/tauri/commands.ts` and a typed schema in `src/lib/schemas.ts`.
  - Drop the `#[allow(dead_code)]` annotations on the same lines as Option A.

- **Recommended:** Option A for v1 (no UI churn, automatic quality, captures
  the spec requirement). Option B if the team wants explicit user control
  over a per-caption review pass.
- **Verify:** With `MockProvider` (no keys), analyze an item image → KA caption
  returned is still the input (mock has no real review, so this exercises the
  best-effort passthrough in `georgian_review`). With a real provider configured
  and a deliberately broken Georgian caption as seed metadata, the second pass
  should improve grammar (manual visual check).

---

## 3. Verification commands (run after each step)

```
cd src-tauri && cargo check                 # type-check
cd src-tauri && cargo test                  # unit tests (migrations, queries, search, prompts::extract_json_object)
npm run check                               # svelte-check + tsc (strict)
npm run build                               # generate build/ for generate_context!
scripts\dev.bat                             # smoke-test full app boot (Tauri + Vite)
```

After 1.1, 1.2, 1.3 ship, run a live end-to-end smoke test:

1. Set `OPENAI_API_KEY` / `GEMINI_API_KEY` / `ANTHROPIC_API_KEY` env vars.
2. `scripts\dev.bat`.
3. In the UI, open Settings → AI → confirm all three providers show
   "configured" status.
4. Switch primary to each provider in turn; on the main screen type a
   Georgian query ("მაცივრის ფასი") and confirm the answer round-trips.
5. Add a product with a photo, click "analyze image" → confirm KA caption is
   natural (not a Russian calque), tags and aliases populated.

---

## 4. Unverified (lowest risk, optional)

### 4.1 `db/queries.rs` body
- Only function signatures were cross-checked against call-sites in
  `ai.rs`, `search/mod.rs`, `seed.rs`. Every usage lined up.
- Action: read `db/queries.rs` end-to-end once before any schema-touching
  work, especially `ai_item_metadata` column mapping.

---

## 5. Explicitly deferred (do not work on in this round)

These are intentional non-goals for the `ai` branch per `CLAUDE.md`:

- `embeddings.rs` — not present; CLAUDE.md marks it as later-phase.
- Full P2P / WebSocket sync — excluded by `sync/mod.rs`'s own comment.
- Migration of `openai.rs` from Chat Completions to the Responses API —
  isolated in `openai.rs`, called out as a next step.
- Cross-provider structured-output unification beyond the per-provider
  pragmatic flags added in §1.

---

## 6. Commit / branch workflow

- One commit per numbered fix in §1 and §2 — keeps the diff reviewable and
  each step individually revertable.
- Commit message format (English, imperative):
  `fix(ai/<provider>): <one-line summary>`.
- After all fixes land on `ai`, push to `origin/AI` from a terminal
  authenticated as a user with write access to `Schreiry/exs`
  (the harness credential `thefatedone` is read-only there).
- Then merge `AI` → `Dev` for integration (Dev is the integration branch per
  `CLAUDE.md`).

---

## 7. Open decisions (need user input before implementation)

1. **§1.4 OpenAI model string** — must be verified against current OpenAI
   model catalog on the day of the change. The dialogue flagged this as
   "exact string to be confirmed at implementation time."
2. **§1.5 Claude structured output** — defer or fix now? See §1.5 for the
   trade-off.
3. **§2.1 Georgian review wiring** — Option A (auto-invoke in
   `analyze_item_image`) or Option B (standalone Tauri command)? Recommended
   is A, but the team may want explicit user control.