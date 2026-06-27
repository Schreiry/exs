-- ============================================================
-- EXSUL (new core): Projection triggers
-- Migration 002 — project item events into the materialized `items` table
--
-- Перенесено из Exsul (002_projection_triggers.sql), сокращено до товарных
-- событий: ItemCreated / ItemUpdated / StockAdjusted / PriceChanged /
-- SaleRecorded. Триггеры — единственное место, где меняется проекция при
-- событийной записи; команды лишь добавляют события.
-- ============================================================

-- ----- ItemCreated -----
CREATE TRIGGER IF NOT EXISTS trg_project_item_created
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

-- ----- ItemUpdated (name / description / category / production_cost / attributes) -----
CREATE TRIGGER IF NOT EXISTS trg_project_item_updated
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

-- ----- StockAdjusted -----
CREATE TRIGGER IF NOT EXISTS trg_project_stock_adjusted
AFTER INSERT ON events
WHEN NEW.event_type = 'StockAdjusted'
BEGIN
    UPDATE items SET
        current_stock = current_stock + COALESCE(json_extract(NEW.data, '$.delta'), 0),
        updated_at = NEW.created_at
    WHERE id = NEW.aggregate_id;
END;

-- ----- PriceChanged -----
CREATE TRIGGER IF NOT EXISTS trg_project_price_changed
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

-- ----- SaleRecorded -----
CREATE TRIGGER IF NOT EXISTS trg_project_sale_recorded
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
