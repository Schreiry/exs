// Scene store — конечный автомат «живого пространства» (Svelte 5 runes).
// Главный экран = состояния одной сцены: idle → searching → results | error.
// Вне Tauri (vite dev в браузере) используется mock, чтобы UI можно было
// смотреть и развивать без бэкенда.

import { assistantQuery, isTauri } from '$lib/tauri/commands';
import type { AssistantResponse, ProductSearchResult } from '$lib/types';

export type SceneMode = 'idle' | 'searching' | 'results' | 'analytics' | 'error';

function detectLang(text: string): 'ru' | 'ka' | 'en' {
	let ka = 0;
	let ru = 0;
	let lat = 0;
	for (const ch of text) {
		const c = ch.codePointAt(0)!;
		if (c >= 0x10a0 && c <= 0x10ff) ka++;
		else if (c >= 0x0400 && c <= 0x04ff) ru++;
		else if (/[a-z]/i.test(ch)) lat++;
	}
	if (ka >= ru && ka >= lat && ka > 0) return 'ka';
	if (ru >= lat && ru > 0) return 'ru';
	return 'en';
}

// Lightweight keyword gate — no AI call, no latency. Detects when the user is
// asking for an analytics / business overview rather than a product search.
const ANALYTICS_PATTERNS: RegExp[] = [
	// Georgian
	/\b(ანალიზ|სტატისტიკ|რეპორტ|ბიზნეს|მარაგ|გაყიდვ|შემოსავალ|რა\s+იყიდება|რა\s+იწვა|რა\s+მარაგი|რა\s+მდგომარეობა)\b/i,
	// Russian
	/\b(анализ|статистик|отчёт|отчет|бизнес|запас|залежал|продаж|доход|выручк)\w*/i,
	// English
	/\b(analytics|statistic|report|business|dead\s*stock|low\s*stock|sales|revenue|overview|insights?)\b/i
];

export function isAnalyticsQuery(text: string): boolean {
	const t = text.trim();
	if (!t) return false;
	return ANALYTICS_PATTERNS.some((re) => re.test(t));
}

class SceneState {
	mode = $state<SceneMode>('idle');
	query = $state('');
	response = $state<AssistantResponse | null>(null);
	error = $state<string | null>(null);
	history = $state<string[]>([]);

	get results(): ProductSearchResult[] {
		return this.response?.search.results ?? [];
	}

	async submit(raw: string) {
		const query = raw.trim();
		if (!query || this.mode === 'searching') return;
		this.query = query;
		this.error = null;

		// Analytics-intent queries bypass the AI gateway and load directly.
		// We mark them as 'searching' briefly so the UI shows the think pulse,
		// then the page flips to 'analytics' on render.
		if (isAnalyticsQuery(query)) {
			this.mode = 'searching';
			// Tiny defer so the pulse paints before the parallel fetches land.
			await new Promise((r) => setTimeout(r, 16));
			this.mode = 'analytics';
			this.history = [query, ...this.history.filter((h) => h !== query)].slice(0, 12);
			return;
		}

		this.mode = 'searching';
		try {
			const resp = isTauri() ? await assistantQuery(query) : mockAssistant(query);
			this.response = resp;
			this.mode = 'results';
			this.history = [query, ...this.history.filter((h) => h !== query)].slice(0, 12);
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
			this.mode = 'error';
		}
	}

	reset() {
		this.mode = 'idle';
		this.response = null;
		this.error = null;
		this.query = '';
	}
}

export const scene = new SceneState();

// — Browser-only mock so the interface is viewable without the Rust backend —
function mockAssistant(query: string): AssistantResponse {
	const lang = detectLang(query);
	const demo: ProductSearchResult[] = [
		mockCard('demo-jeep', 'სათამაშო ჯიპი', 'სათამაშოები', 45, 12, 'visual', ['name', 'aliases']),
		mockCard('demo-basket', 'სასაჩუქრე კალათა', 'საჩუქრები', 120, 6, 'primary', ['category']),
		mockCard('demo-tea', 'მწვანე ჩაი', 'სასმელები', 18.5, 30, 'quiet', ['ai_tags'])
	];
	const summary =
		lang === 'ka'
			? `ნაპოვნია ${demo.length} შედეგი მოთხოვნაზე «${query}».`
			: lang === 'ru'
				? `Найдено результатов: ${demo.length} по запросу «${query}».`
				: `Found ${demo.length} result(s) for “${query}”.`;
	return {
		mode: 'assistant',
		language: lang,
		search: { mode: 'product_search', query, language: lang, results: demo, assistant_summary: summary },
		answer: { text: `(mock) ${summary}`, language: lang, provider: 'mock' },
		answer_error: null
	};
}

function mockCard(
	id: string,
	name: string,
	category: string,
	price: number,
	stock: number,
	hint: ProductSearchResult['card_style_hint'],
	matched: string[]
): ProductSearchResult {
	return {
		item_id: id,
		title: name,
		reason: `ემთხვევა: ${matched.join(', ')}`,
		confidence: 0.6,
		matched_by: matched,
		card_style_hint: hint,
		item: {
			id,
			name,
			description: '',
			category,
			category_id: null,
			initial_price: price,
			current_price: price,
			production_cost: 0,
			current_stock: stock,
			sold_count: 0,
			revenue: 0,
			attributes_json: '{}',
			image_path: null,
			card_color: null,
			created_at: '',
			updated_at: ''
		}
	};
}
