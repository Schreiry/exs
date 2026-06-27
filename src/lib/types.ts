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

export interface ItemImage {
	mime: string;
	base64: string;
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

// ── Explicit local AI context ──────────────────────────────────

export type ContextFileKind =
	| 'plain_text'
	| 'markdown'
	| 'delimited_data'
	| 'json'
	| 'yaml'
	| 'toml'
	| 'xml'
	| 'html'
	| 'configuration'
	| 'sql'
	| 'source_code';

export type ContextFileErrorCode =
	| 'dialog_failed'
	| 'unsupported_path'
	| 'unsupported_type'
	| 'not_a_file'
	| 'file_too_large'
	| 'file_unavailable'
	| 'invalid_encoding'
	| 'binary_content'
	| 'not_selected'
	| 'selection_limit'
	| 'registry_full'
	| 'internal';

export interface ContextFileIssue {
	code: ContextFileErrorCode;
	message: string;
	file_name: string;
}

export interface ContextFileMetadata {
	/** Opaque, session-only capability. This is intentionally not a path. */
	selection_id: string;
	file_name: string;
	extension: string;
	kind: ContextFileKind;
	size_bytes: number;
	modified_at: string | null;
}

export interface ContextFileSelection {
	cancelled: boolean;
	files: ContextFileMetadata[];
	rejected: ContextFileIssue[];
	max_file_bytes: number;
}

export interface ContextFileDocument {
	file: ContextFileMetadata;
	content: string;
	content_sha256: string;
}
