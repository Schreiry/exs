-- ============================================================
-- EXSUL (new core): AI metadata
-- Migration 003 — per-item AI-generated metadata
--
-- Дополнительная таблица, НЕ ломающая существующие данные. Хранит
-- мультиязычные подписи к фото, теги, визуальные атрибуты, алиасы
-- (RU/KA/EN морфологические варианты) и состояние эмбеддингов.
-- ============================================================

CREATE TABLE IF NOT EXISTS ai_item_metadata (
    item_id              TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    image_caption_ru     TEXT,
    image_caption_ka     TEXT,
    image_caption_en     TEXT,
    tags_json            TEXT NOT NULL DEFAULT '[]',
    visual_attributes_json TEXT NOT NULL DEFAULT '{}',
    aliases_json         TEXT NOT NULL DEFAULT '[]',
    embedding_model      TEXT,
    embedding_updated_at TEXT,
    ai_updated_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
