import type {
	ActionDefinition,
	ActionIslandOpenReason,
	ActionRegistration,
	RegisterActionOptions
} from './types';

function assertAction(action: ActionDefinition): void {
	if (!action.id.trim()) throw new Error('Action id must not be empty.');
	if (!action.label.trim()) throw new Error(`Action "${action.id}" must have a label.`);
	if (typeof action.run !== 'function') throw new Error(`Action "${action.id}" must have a run handler.`);
}

function isVisible(action: ActionDefinition): boolean {
	return typeof action.visible === 'function' ? action.visible() : action.visible !== false;
}

/**
 * Reactive registry shared by independent feature modules.
 *
 * Register during module setup/onMount and retain the returned cleanup function:
 * `const registration = actionRegistry.register(action);`
 * `onDestroy(registration.unregister);`
 */
export class ActionRegistry {
	#entries = $state.raw(new Map<string, ActionDefinition>());

	get actions(): readonly ActionDefinition[] {
		return [...this.#entries.values()]
			.filter(isVisible)
			.sort((a, b) => (a.order ?? 0) - (b.order ?? 0) || a.label.localeCompare(b.label));
	}

	get(id: string): ActionDefinition | undefined {
		return this.#entries.get(id);
	}

	register(
		action: ActionDefinition,
		options: RegisterActionOptions = {}
	): ActionRegistration {
		assertAction(action);
		const id = action.id.trim();
		const current = this.#entries.get(id);
		if (current && !options.replace) {
			throw new Error(`Action "${id}" is already registered.`);
		}

		const registered: ActionDefinition = { ...action, id };
		const next = new Map(this.#entries);
		next.set(id, registered);
		this.#entries = next;

		let active = true;
		return {
			id,
			unregister: () => {
				if (!active) return;
				active = false;
				// A stale disposer must never remove a newer replacement.
				if (this.#entries.get(id) !== registered) return;
				const updated = new Map(this.#entries);
				updated.delete(id);
				this.#entries = updated;
			}
		};
	}

	registerMany(
		actions: readonly ActionDefinition[],
		options: RegisterActionOptions = {}
	): () => void {
		const ids = new Set<string>();
		for (const action of actions) {
			assertAction(action);
			const id = action.id.trim();
			if (ids.has(id)) throw new Error(`Action "${id}" occurs more than once in this batch.`);
			if (this.#entries.has(id) && !options.replace) {
				throw new Error(`Action "${id}" is already registered.`);
			}
			ids.add(id);
		}

		const registrations = actions.map((action) => this.register(action, options));
		return () => {
			for (const registration of registrations) registration.unregister();
		};
	}
}

export class ActionIslandController {
	isOpen = $state(false);
	openReason = $state<ActionIslandOpenReason>('api');

	open(reason: ActionIslandOpenReason = 'api'): void {
		this.openReason = reason;
		this.isOpen = true;
	}

	close(): void {
		this.isOpen = false;
	}

	toggle(reason: ActionIslandOpenReason = 'api'): void {
		if (this.isOpen) this.close();
		else this.open(reason);
	}
}

export const actionRegistry = new ActionRegistry();
export const actionIsland = new ActionIslandController();

/** Preserves literal ids and metadata while type-checking feature-owned actions. */
export function defineAction<const T extends ActionDefinition>(action: T): T {
	return action;
}

export function registerAction(
	action: ActionDefinition,
	options?: RegisterActionOptions
): ActionRegistration {
	return actionRegistry.register(action, options);
}

export function registerActions(
	actions: readonly ActionDefinition[],
	options?: RegisterActionOptions
): () => void {
	return actionRegistry.registerMany(actions, options);
}
