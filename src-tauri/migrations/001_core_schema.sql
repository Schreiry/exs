-- ============================================================
-- EXSUL (new core): Event-sourced inventory schema
-- Migration 001 — base schema
--
-- Перенесено из Exsul: событийный леджер (events) + материализованная
-- проекция (items). Старый доменный слой (flowers/orders/greenhouse/...)
-- НЕ переносился — это чистое ядро под AI-ассистента.
-- ============================================================

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ===================
-- EVENT LEDGER (append-only, single source of truth)
-- ===================
CREATE TABLE IF NOT EXISTS events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    aggregate_id    TEXT    NOT NULL,
    aggregate_type  TEXT    NOT NULL,
    event_type      TEXT    NOT NULL,
    data            TEXT    NOT NULL DEFAULT '{}',
    hlc_timestamp   TEXT    NOT NULL,
    node_id         TEXT    NOT NULL,
    version         INTEGER NOT NULL,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),

    UNIQUE(aggregate_id, node_id, version)
);

CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_id, version);
CREATE INDEX IF NOT EXISTS idx_events_hlc ON events(hlc_timestamp);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);

-- ===================
-- PROJECTION: Items (current state of every product/card)
-- ===================
CREATE TABLE IF NOT EXISTS items (
    id               TEXT    PRIMARY KEY,
    name             TEXT    NOT NULL,
    description      TEXT    NOT NULL DEFAULT '',
    category         TEXT    NOT NULL DEFAULT 'uncategorized',
    category_id      TEXT,
    initial_price    REAL    NOT NULL DEFAULT 0.0,
    current_price    REAL    NOT NULL DEFAULT 0.0,
    production_cost  REAL    NOT NULL DEFAULT 0.0,
    current_stock    INTEGER NOT NULL DEFAULT 0,
    sold_count       INTEGER NOT NULL DEFAULT 0,
    revenue          REAL    NOT NULL DEFAULT 0.0,
    -- structured visual/business attributes (color, material, size, ...) as JSON
    attributes_json  TEXT    NOT NULL DEFAULT '{}',
    image_path       TEXT,
    card_color       TEXT,
    created_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now')),
    updated_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_items_category ON items(category);
CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);
CREATE INDEX IF NOT EXISTS idx_items_updated ON items(updated_at);

-- ===================
-- PROJECTION: Price history
-- ===================
CREATE TABLE IF NOT EXISTS item_prices (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id       TEXT    NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    price         REAL    NOT NULL,
    effective_at  TEXT    NOT NULL,
    event_id      INTEGER REFERENCES events(id),
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_item_prices_item ON item_prices(item_id, effective_at);

-- ===================
-- Product photos (multiple per item; items.image_path = primary thumbnail)
-- ===================
CREATE TABLE IF NOT EXISTS item_photos (
    id          TEXT    PRIMARY KEY,
    item_id     TEXT    NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    path        TEXT    NOT NULL,
    is_primary  INTEGER NOT NULL DEFAULT 0,
    sort_index  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_item_photos_item ON item_photos(item_id, sort_index);

-- ===================
-- Categories
-- ===================
CREATE TABLE IF NOT EXISTS categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    color       TEXT,
    icon        TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

-- ===================
-- Sync state (vector clock per peer) — kept for future P2P, harmless if unused
-- ===================
CREATE TABLE IF NOT EXISTS sync_state (
    peer_node_id     TEXT    PRIMARY KEY,
    last_hlc         TEXT    NOT NULL,
    last_event_id    INTEGER NOT NULL DEFAULT 0,
    last_synced_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

-- ===================
-- Local config (node_id, recovery state, AI provider selection, ...)
-- ===================
CREATE TABLE IF NOT EXISTS local_config (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
);

-- ===================
-- App settings (user-facing key/value, JSON values)
-- ===================
CREATE TABLE IF NOT EXISTS app_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL DEFAULT '{}',
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

-- ===================
-- Audit log (human-readable trail; secrets are NEVER written here)
-- ===================
CREATE TABLE IF NOT EXISTS audit_logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     TEXT NOT NULL DEFAULT 'local',
    action      TEXT NOT NULL,
    payload     TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at);
