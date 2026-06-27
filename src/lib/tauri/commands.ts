// Typed wrappers вокруг Tauri-команд. Единственное место, где фронтенд вызывает
// бэкенд: имена команд, типы аргументов/результата и Zod-валидация на границе.

import { invoke } from '@tauri-apps/api/core';
import {
	AssistantResponseSchema,
	ProductSearchResponseSchema,
	AiStatusSchema,
	InventorySummarySchema,
	TopSellerSchema,
	DeadStockSchema,
	LowStockItemSchema,
	CategoryBreakdownSchema,
	TimeseriesPointSchema,
	AiCoverageSchema,
	ActivityEntrySchema,
	StockOutForecastSchema,
	HeatmapCellSchema
} from '$lib/schemas';
import type {
	AssistantResponse,
	ProductSearchResponse,
	AiStatus,
	AiItemMetadata,
	InventorySummary,
	Item,
	ListPage,
	CreateItemPayload,
	RestoreReport,
	TopSeller,
	DeadStock,
	LowStockItem,
	CategoryBreakdown,
	TimeseriesPoint,
	AiCoverage,
	ActivityEntry,
	StockOutForecast,
	HeatmapCell,
	SalesBucket
} from '$lib/types';

/** Detect whether we're running inside the Tauri webview (vs plain browser dev). */
export function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// ── AI / assistant ──────────────────────────────────────────────

export async function assistantQuery(
	query: string,
	language?: 'ru' | 'ka' | 'en'
): Promise<AssistantResponse> {
	const raw = await invoke('assistant_query', { query, language });
	return AssistantResponseSchema.parse(raw) as AssistantResponse;
}

export async function searchProducts(
	query: string,
	limit?: number
): Promise<ProductSearchResponse> {
	const raw = await invoke('search_products', { query, limit });
	return ProductSearchResponseSchema.parse(raw) as ProductSearchResponse;
}

export async function analyzeItemImage(itemId: string): Promise<AiItemMetadata> {
	return (await invoke('analyze_item_image', { itemId })) as AiItemMetadata;
}

export async function aiGetStatus(): Promise<AiStatus> {
	const raw = await invoke('ai_get_status');
	return AiStatusSchema.parse(raw) as AiStatus;
}

export async function aiSetProvider(provider: string): Promise<void> {
	await invoke('ai_set_provider', { provider });
}

export async function aiSetProviderKey(provider: string, key: string): Promise<void> {
	await invoke('ai_set_provider_key', { provider, key });
}

export async function aiDeleteProviderKey(provider: string): Promise<void> {
	await invoke('ai_delete_provider_key', { provider });
}

// ── Inventory ───────────────────────────────────────────────────

export async function getItems(limit?: number): Promise<ListPage<Item>> {
	return (await invoke('get_items', { limit })) as ListPage<Item>;
}

export async function addItem(payload: CreateItemPayload): Promise<string> {
	return (await invoke('add_item', { payload })) as string;
}

export async function saveItemImage(itemId: string, base64Data: string): Promise<string> {
	return (await invoke('save_item_image', { itemId, base64Data })) as string;
}

export async function seedDemoItems(): Promise<number> {
	return (await invoke('seed_demo_items')) as number;
}

export async function getInventorySummary(lowStockThreshold?: number): Promise<InventorySummary> {
	const raw = await invoke('get_inventory_summary', { lowStockThreshold });
	return InventorySummarySchema.parse(raw) as InventorySummary;
}

export async function rebuildSearchIndex(): Promise<number> {
	return (await invoke('rebuild_search_index')) as number;
}

// ── Backup / system ─────────────────────────────────────────────

export async function exportBackup(): Promise<string> {
	return (await invoke('export_backup')) as string;
}

export async function importBackup(path: string): Promise<RestoreReport> {
	return (await invoke('import_backup', { path })) as RestoreReport;
}

export async function getAppVersion(): Promise<string> {
	return (await invoke('get_app_version')) as string;
}

// ── Analytics ─────────────────────────────────────────────────────

export async function getTopSellers(limit?: number): Promise<TopSeller[]> {
	const raw = await invoke<TopSeller[]>('get_top_sellers', { limit });
	return raw.map((r) => TopSellerSchema.parse(r));
}

export async function getDeadStock(limit?: number): Promise<DeadStock[]> {
	const raw = await invoke<DeadStock[]>('get_dead_stock', { limit });
	return raw.map((r) => DeadStockSchema.parse(r));
}

export async function getLowStockItems(
	threshold?: number,
	limit?: number
): Promise<LowStockItem[]> {
	const raw = await invoke<LowStockItem[]>('get_low_stock_items', { threshold, limit });
	return raw.map((r) => LowStockItemSchema.parse(r));
}

export async function getCategoryBreakdown(): Promise<CategoryBreakdown[]> {
	const raw = await invoke<CategoryBreakdown[]>('get_category_breakdown');
	return raw.map((r) => CategoryBreakdownSchema.parse(r));
}

export async function getSalesTimeseries(
	bucket: SalesBucket = 'day',
	since?: string
): Promise<TimeseriesPoint[]> {
	const raw = await invoke<TimeseriesPoint[]>('get_sales_timeseries', { bucket, since });
	return raw.map((r) => TimeseriesPointSchema.parse(r));
}

export async function getAiCoverage(): Promise<AiCoverage> {
	const raw = await invoke('get_ai_coverage');
	return AiCoverageSchema.parse(raw) as AiCoverage;
}

export async function getRecentActivity(limit?: number): Promise<ActivityEntry[]> {
	const raw = await invoke<ActivityEntry[]>('get_recent_activity', { limit });
	return raw.map((r) => ActivityEntrySchema.parse(r));
}

export async function getStockOutForecast(
	limit?: number,
	minSales?: number
): Promise<StockOutForecast[]> {
	const raw = await invoke<StockOutForecast[]>('get_stock_out_forecast', {
		limit,
		minSales
	});
	return raw.map((r) => StockOutForecastSchema.parse(r));
}

export async function getSalesHeatmap(since?: string): Promise<HeatmapCell[]> {
	const raw = await invoke<HeatmapCell[]>('get_sales_heatmap', { since });
	return raw.map((r) => HeatmapCellSchema.parse(r));
}
