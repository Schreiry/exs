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

export const ItemImageSchema = z.object({
	mime: z.string(),
	base64: z.string()
});

export const ContextFileKindSchema = z.enum([
	'plain_text',
	'markdown',
	'delimited_data',
	'json',
	'yaml',
	'toml',
	'xml',
	'html',
	'configuration',
	'sql',
	'source_code'
]);

export const ContextFileErrorCodeSchema = z.enum([
	'dialog_failed',
	'unsupported_path',
	'unsupported_type',
	'not_a_file',
	'file_too_large',
	'file_unavailable',
	'invalid_encoding',
	'binary_content',
	'not_selected',
	'selection_limit',
	'registry_full',
	'internal'
]);

export const ContextFileMetadataSchema = z.object({
	selection_id: z.string().uuid(),
	file_name: z.string(),
	extension: z.string(),
	kind: ContextFileKindSchema,
	size_bytes: z.number().int().nonnegative(),
	modified_at: z.string().datetime({ offset: true }).nullable()
});

export const ContextFileSelectionSchema = z.object({
	cancelled: z.boolean(),
	files: z.array(ContextFileMetadataSchema),
	rejected: z.array(
		z.object({
			code: ContextFileErrorCodeSchema,
			message: z.string(),
			file_name: z.string()
		})
	),
	max_file_bytes: z.number().int().positive()
});

export const ContextFileDocumentSchema = z.object({
	file: ContextFileMetadataSchema,
	content: z.string(),
	content_sha256: z.string().regex(/^[a-f0-9]{64}$/)
});
