-- ============================================================
-- EXSUL (new core): legacy database compatibility bridge
-- Migration 1001
--
-- Older Exsul installations already used schema_migrations versions 1..32.
-- Consequently, the new standalone core migrations 001..004 are skipped on
-- those databases. This high-numbered, idempotent bridge fills only the
-- columns/tables required by the new core and preserves every legacy row.
-- ============================================================

-- `ALTER TABLE ... ADD COLUMN` has no portable IF NOT EXISTS in SQLite.
-- The migration runner deliberately retries statement-by-statement and skips
-- duplicate-column errors, making these additions safe on both old and fresh
-- databases.
ALTER TABLE items ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE items ADD COLUMN category TEXT NOT NULL DEFAULT 'uncategorized';
ALTER TABLE items ADD COLUMN category_id TEXT;
ALTER TABLE items ADD COLUMN initial_price REAL NOT NULL DEFAULT 0.0;
ALTER TABLE items ADD COLUMN current_price REAL NOT NULL DEFAULT 0.0;
ALTER TABLE items ADD COLUMN production_cost REAL NOT NULL DEFAULT 0.0;
ALTER TABLE items ADD COLUMN current_stock INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN sold_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN revenue REAL NOT NULL DEFAULT 0.0;
ALTER TABLE items ADD COLUMN attributes_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE items ADD COLUMN image_path TEXT;
ALTER TABLE items ADD COLUMN card_color TEXT;
ALTER TABLE items ADD COLUMN created_at TEXT NOT NULL DEFAULT '';
ALTER TABLE items ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';

UPDATE items
SET created_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
WHERE created_at = '';

UPDATE items
SET updated_at = COALESCE(NULLIF(created_at, ''), strftime('%Y-%m-%dT%H:%M:%f', 'now'))
WHERE updated_at = '';

CREATE INDEX IF NOT EXISTS idx_items_category ON items(category);
CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);
CREATE INDEX IF NOT EXISTS idx_items_updated ON items(updated_at);

CREATE TABLE IF NOT EXISTS item_photos (
    id          TEXT    PRIMARY KEY,
    item_id     TEXT    NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    path        TEXT    NOT NULL,
    is_primary  INTEGER NOT NULL DEFAULT 0,
    sort_index  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_item_photos_item ON item_photos(item_id, sort_index);

CREATE TABLE IF NOT EXISTS ai_item_metadata (
    item_id                   TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    image_caption_ru          TEXT,
    image_caption_ka          TEXT,
    image_caption_en          TEXT,
    tags_json                 TEXT NOT NULL DEFAULT '[]',
    visual_attributes_json    TEXT NOT NULL DEFAULT '{}',
    aliases_json              TEXT NOT NULL DEFAULT '[]',
    embedding_model           TEXT,
    embedding_updated_at      TEXT,
    ai_updated_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

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

-- Replace legacy projection triggers only after all bridge columns exist.
DROP TRIGGER IF EXISTS trg_project_item_created;
DROP TRIGGER IF EXISTS trg_project_item_updated;
DROP TRIGGER IF EXISTS trg_project_stock_adjusted;
DROP TRIGGER IF EXISTS trg_project_price_changed;
DROP TRIGGER IF EXISTS trg_project_sale_recorded;

CREATE TRIGGER trg_project_item_created
AFTER INSERT ON events
WHEN NEW.event_type = 'ItemCreated'
BEGIN
    INSERT OR REPLACE INTO items (
        id, name, description, category, category_id,
        initial_price, current_price, production_cost,
        current_stock, sold_count, revenue, attributes_json,
        created_at, updated_at
    ) VALUES (
        NEW.aggregate_id,
        json_extract(NEW.data, '$.name'),
        COALESCE(json_extract(NEW.data, '$.description'), ''),
        COALESCE(json_extract(NEW.data, '$.category'), 'uncategorized'),
        json_extract(NEW.data, '$.category_id'),
        COALESCE(json_extract(NEW.data, '$.price'), 0.0),
        COALESCE(json_extract(NEW.data, '$.price'), 0.0),
        COALESCE(json_extract(NEW.data, '$.production_cost'), 0.0),
        COALESCE(json_extract(NEW.data, '$.initial_stock'), 0),
        0,
        0.0,
        COALESCE(json_extract(NEW.data, '$.attributes_json'), '{}'),
        NEW.created_at,
        NEW.created_at
    );

    INSERT INTO item_prices (item_id, price, effective_at, event_id)
    VALUES (
        NEW.aggregate_id,
        COALESCE(json_extract(NEW.data, '$.price'), 0.0),
        NEW.hlc_timestamp,
        NEW.id
    );
END;

CREATE TRIGGER trg_project_item_updated
AFTER INSERT ON events
WHEN NEW.event_type = 'ItemUpdated'
BEGIN
    UPDATE items SET
        name            = COALESCE(json_extract(NEW.data, '$.name'), name),
        description     = COALESCE(json_extract(NEW.data, '$.description'), description),
        category        = COALESCE(json_extract(NEW.data, '$.category'), category),
        category_id     = COALESCE(json_extract(NEW.data, '$.category_id'), category_id),
        production_cost = COALESCE(json_extract(NEW.data, '$.production_cost'), production_cost),
        attributes_json = COALESCE(json_extract(NEW.data, '$.attributes_json'), attributes_json),
        updated_at      = NEW.created_at
    WHERE id = NEW.aggregate_id;
END;

CREATE TRIGGER trg_project_stock_adjusted
AFTER INSERT ON events
WHEN NEW.event_type = 'StockAdjusted'
BEGIN
    UPDATE items SET
        current_stock = current_stock + COALESCE(json_extract(NEW.data, '$.delta'), 0),
        updated_at = NEW.created_at
    WHERE id = NEW.aggregate_id;
END;

CREATE TRIGGER trg_project_price_changed
AFTER INSERT ON events
WHEN NEW.event_type = 'PriceChanged'
BEGIN
    UPDATE items SET
        current_price = json_extract(NEW.data, '$.new_price'),
        updated_at = NEW.created_at
    WHERE id = NEW.aggregate_id;

    INSERT INTO item_prices (item_id, price, effective_at, event_id)
    VALUES (
        NEW.aggregate_id,
        json_extract(NEW.data, '$.new_price'),
        NEW.hlc_timestamp,
        NEW.id
    );
END;

CREATE TRIGGER trg_project_sale_recorded
AFTER INSERT ON events
WHEN NEW.event_type = 'SaleRecorded'
BEGIN
    UPDATE items SET
        current_stock = current_stock - COALESCE(json_extract(NEW.data, '$.quantity'), 1),
        sold_count = sold_count + COALESCE(json_extract(NEW.data, '$.quantity'), 1),
        revenue = revenue + (
            COALESCE(json_extract(NEW.data, '$.sale_price'), current_price)
            * COALESCE(json_extract(NEW.data, '$.quantity'), 1)
        ),
        updated_at = NEW.created_at
    WHERE id = NEW.aggregate_id;
END;

-- Existing products become searchable immediately after the first upgraded
-- launch; users do not need to trigger a manual index rebuild.
DELETE FROM item_search_fts;

INSERT INTO item_search_fts (
    item_id, name, description, category, ai_tags, ai_caption, aliases
)
SELECT
    i.id,
    i.name,
    i.description,
    i.category,
    COALESCE(m.tags_json, ''),
    COALESCE(m.image_caption_ka, '') || ' '
        || COALESCE(m.image_caption_ru, '') || ' '
        || COALESCE(m.image_caption_en, ''),
    COALESCE(m.aliases_json, '')
FROM items i
LEFT JOIN ai_item_metadata m ON m.item_id = i.id;
