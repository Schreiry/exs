// Tiny i18n helper. Приложение по умолчанию на грузинском (ka). Хелпер t()
// резолвит точечный путь по словарю и подставляет {placeholders}.

import { ka } from './ka';

const dicts = { ka } as const;
export type Locale = keyof typeof dicts;

// Текущая локаль приложения — грузинский.
let current: Locale = 'ka';

export function setLocale(locale: Locale) {
	current = locale;
}

/** Resolve a dotted key path against the active dictionary. */
export function t(path: string, params?: Record<string, string | number>): string {
	const parts = path.split('.');
	let node: unknown = dicts[current];
	for (const p of parts) {
		if (node && typeof node === 'object' && p in (node as Record<string, unknown>)) {
			node = (node as Record<string, unknown>)[p];
		} else {
			return path; // missing key → echo path (visible in dev)
		}
	}
	let str = typeof node === 'string' ? node : path;
	if (params) {
		for (const [k, v] of Object.entries(params)) {
			str = str.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
		}
	}
	return str;
}
