// Zod-схемы для runtime-валидации данных, приходящих от Tauri-команд.
// invoke() возвращает `unknown` по сути — валидируем на границе, чтобы UI
// не падал на неожиданной форме.

import { z } from 'zod';

export const ItemSchema = z.object({
	id: z.string(),
	name: z.string(),
	description: z.string(),
	category: z.string(),
	category_id: z.string().nullable(),
	initial_price: z.number(),
	current_price: z.number(),
	production_cost: z.number(),
	current_stock: z.number(),
	sold_count: z.number(),
	revenue: z.number(),
	attributes_json: z.string(),
	image_path: z.string().nullable(),
	card_color: z.string().nullable(),
	created_at: z.string(),
	updated_at: z.string()
});

export const ProductSearchResultSchema = z.object({
	item_id: z.string(),
	title: z.string(),
	reason: z.string(),
	confidence: z.number(),
	matched_by: z.array(z.string()),
	card_style_hint: z.enum(['primary', 'quiet', 'visual', 'dense']),
	item: ItemSchema
});

export const ProductSearchResponseSchema = z.object({
	mode: z.literal('product_search'),
	query: z.string(),
	language: z.enum(['ru', 'ka', 'en']),
	results: z.array(ProductSearchResultSchema),
	assistant_summary: z.string()
});

export const AiAnswerSchema = z.object({
	text: z.string(),
	language: z.string(),
	provider: z.string()
});

export const AssistantResponseSchema = z.object({
	mode: z.literal('assistant'),
	language: z.enum(['ru', 'ka', 'en']),
	search: ProductSearchResponseSchema,
	answer: AiAnswerSchema.nullable(),
	answer_error: z.string().nullable()
});

export const AiStatusSchema = z.object({
	selected: z.string(),
	providers: z.array(
		z.object({
			provider: z.string(),
			configured: z.boolean(),
			is_primary: z.boolean()
		})
	)
});

export const InventorySummarySchema = z.object({
	item_count: z.number(),
	total_stock_units: z.number(),
	stock_value_at_price: z.number(),
	stock_value_at_cost: z.number(),
	total_revenue: z.number(),
	total_sold: z.number(),
	low_stock_count: z.number(),
	top_categories: z.array(z.object({ category: z.string(), count: z.number() }))
});

// ── Analytics ─────────────────────────────────────────────────────

export const TopSellerSchema = z.object({
	id: z.string(),
	name: z.string(),
	category: z.string(),
	sold_count: z.number(),
	revenue: z.number(),
	current_price: z.number(),
	current_stock: z.number()
});

export const DeadStockSchema = z.object({
	id: z.string(),
	name: z.string(),
	category: z.string(),
	current_stock: z.number(),
	current_price: z.number(),
	locked_value: z.number()
});

export const LowStockItemSchema = z.object({
	id: z.string(),
	name: z.string(),
	category: z.string(),
	current_stock: z.number(),
	current_price: z.number()
});

export const CategoryBreakdownSchema = z.object({
	category: z.string(),
	item_count: z.number(),
	stock_units: z.number(),
	stock_value: z.number(),
	revenue: z.number(),
	sold_count: z.number()
});

export const TimeseriesPointSchema = z.object({
	bucket: z.string(),
	sales_count: z.number(),
	units_sold: z.number(),
	revenue: z.number()
});

export const AiCoverageSchema = z.object({
	total_items: z.number(),
	with_caption_ka: z.number(),
	with_tags: z.number(),
	with_aliases: z.number(),
	latest_ai_update: z.string().nullable()
});

export const ActivityEntrySchema = z.object({
	event_type: z.string(),
	item_id: z.string(),
	item_name: z.string(),
	summary: z.string(),
	hlc_timestamp: z.string(),
	created_at: z.string().nullable()
});

export const StockOutForecastSchema = z.object({
	id: z.string(),
	name: z.string(),
	category: z.string(),
	current_stock: z.number(),
	sold_count: z.number(),
	velocity_per_day: z.number(),
	days_until_stockout: z.number(),
	first_sale_at: z.string().nullable(),
	history_days: z.number()
});

export const HeatmapCellSchema = z.object({
	weekday: z.number().int().min(0).max(6),
	hour: z.number().int().min(0).max(23),
	revenue: z.number(),
	units: z.number()
});
