// TypeScript interfaces mirroring the Rust structs (events::types / search / ai).
// Эти типы — контракт фронтенда с typed Tauri-командами. Держим их в синхроне
// с src-tauri (serde). Runtime-валидация — в schemas.ts (Zod).

export interface Item {
	id: string;
	name: string;
	description: string;
	category: string;
	category_id: string | null;
	initial_price: number;
	current_price: number;
	production_cost: number;
	current_stock: number;
	sold_count: number;
	revenue: number;
	attributes_json: string;
	image_path: string | null;
	card_color: string | null;
	created_at: string;
	updated_at: string;
}

export interface ListPage<T> {
	rows: T[];
	truncated: boolean;
}

export type CardStyleHint = 'primary' | 'quiet' | 'visual' | 'dense';

export interface ProductSearchResult {
	item_id: string;
	title: string;
	reason: string;
	confidence: number;
	matched_by: string[];
	card_style_hint: CardStyleHint;
	item: Item;
}

export interface ProductSearchResponse {
	mode: 'product_search';
	query: string;
	language: 'ru' | 'ka' | 'en';
	results: ProductSearchResult[];
	assistant_summary: string;
}

export interface AiAnswer {
	text: string;
	language: string;
	provider: string;
}

export interface AssistantResponse {
	mode: 'assistant';
	language: 'ru' | 'ka' | 'en';
	search: ProductSearchResponse;
	answer: AiAnswer | null;
	answer_error: string | null;
}

export interface AiItemMetadata {
	item_id: string;
	image_caption_ru: string | null;
	image_caption_ka: string | null;
	image_caption_en: string | null;
	tags: string[];
	visual_attributes: unknown;
	aliases: string[];
	embedding_model: string | null;
	embedding_updated_at: string | null;
	ai_updated_at: string | null;
}

export interface ProviderStatus {
	provider: string;
	configured: boolean;
	is_primary: boolean;
}

export interface AiStatus {
	selected: string;
	providers: ProviderStatus[];
}

export interface CategoryCount {
	category: string;
	count: number;
}

export interface InventorySummary {
	item_count: number;
	total_stock_units: number;
	stock_value_at_price: number;
	stock_value_at_cost: number;
	total_revenue: number;
	total_sold: number;
	low_stock_count: number;
	top_categories: CategoryCount[];
}

export interface Category {
	id: string;
	name: string;
	color: string | null;
	icon: string | null;
	created_at: string;
}

export interface RestoreReport {
	items_restored: number;
	images_restored: number;
	message: string;
}

// ── Analytics (mirror of events::types extended analytics) ──────────

export interface TopSeller {
	id: string;
	name: string;
	category: string;
	sold_count: number;
	revenue: number;
	current_price: number;
	current_stock: number;
}

export interface DeadStock {
	id: string;
	name: string;
	category: string;
	current_stock: number;
	current_price: number;
	locked_value: number;
}

export interface LowStockItem {
	id: string;
	name: string;
	category: string;
	current_stock: number;
	current_price: number;
}

export interface CategoryBreakdown {
	category: string;
	item_count: number;
	stock_units: number;
	stock_value: number;
	revenue: number;
	sold_count: number;
}

export type SalesBucket = 'day' | 'week' | 'month';

export interface TimeseriesPoint {
	bucket: string;
	sales_count: number;
	units_sold: number;
	revenue: number;
}

export interface AiCoverage {
	total_items: number;
	with_caption_ka: number;
	with_tags: number;
	with_aliases: number;
	latest_ai_update: string | null;
}

export interface ActivityEntry {
	event_type: string;
	item_id: string;
	item_name: string;
	summary: string;
	hlc_timestamp: string;
	created_at: string | null;
}

export interface StockOutForecast {
	id: string;
	name: string;
	category: string;
	current_stock: number;
	sold_count: number;
	velocity_per_day: number;
	days_until_stockout: number;
	first_sale_at: string | null;
	history_days: number;
}

export interface HeatmapCell {
	/** 0 = Sunday … 6 = Saturday */
	weekday: number;
	/** 0..23 */
	hour: number;
	revenue: number;
	units: number;
}

export type HeatmapPeriodDays = 7 | 30 | 90 | 0; // 0 = all time

export interface CreateItemPayload {
	name: string;
	description?: string;
	category?: string;
	category_id?: string;
	price?: number;
	production_cost?: number;
	initial_stock?: number;
	attributes?: unknown;
}
