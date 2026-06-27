-- ============================================================
-- EXSUL (new core): Full-text search
-- Migration 004 — FTS5 index over product + AI fields
--
-- Standalone FTS5 table (не external-content), чтобы можно было склеивать
-- поля из двух источников: items (name/description/category) и
-- ai_item_metadata (tags/caption/aliases). Индекс обслуживается из Rust
-- (см. db::queries::reindex_item_fts) — это надёжнее, чем хрупкие
-- мультитабличные триггеры. item_id хранится как UNINDEXED для удаления.
-- ============================================================

CREATE VIRTUAL TABLE IF NOT EXISTS item_search_fts USING fts5(
    item_id UNINDEXED,
    name,
    description,
    category,
    ai_tags,
    ai_caption,
    aliases,
    tokenize = 'unicode61 remove_diacritics 2'
);
