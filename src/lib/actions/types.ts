export type ActionTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
export type ActionDismissMode = 'immediate' | 'after-run' | 'manual';
export type ActionTriggerSource = 'pointer' | 'keyboard';
export type ActionIslandOpenReason = 'api' | 'hotkey' | 'shake';

export interface ActionExecutionContext {
	/** How the user launched this action. */
	source: ActionTriggerSource;
	/** Aborted when the island closes or another action starts (except detached `immediate` actions). */
	signal: AbortSignal;
	/** Lets long-running or multi-step actions close the island explicitly. */
	close: () => void;
}

export interface ActionDefinition {
	/**
	 * Stable, namespaced identifier, for example `ai.summarize` or
	 * `analytics.open-overview`.
	 */
	id: string;
	label: string;
	description?: string;
	/** A short glyph, emoji or text mark. Decorative; label remains the accessible name. */
	icon?: string;
	group?: string;
	badge?: string | (() => string | undefined);
	keywords?: readonly string[];
	order?: number;
	tone?: ActionTone;
	enabled?: boolean | (() => boolean);
	visible?: boolean | (() => boolean);
	dismiss?: ActionDismissMode;
	run: (context: ActionExecutionContext) => void | Promise<void>;
}

export interface RegisterActionOptions {
	/**
	 * Replace an existing action with the same id. Prefer the default (`false`)
	 * outside hot-module-reload adapters so branch integration catches collisions.
	 */
	replace?: boolean;
}

export interface ActionRegistration {
	readonly id: string;
	unregister: () => void;
}
