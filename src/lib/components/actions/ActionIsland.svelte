<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import {
		actionIsland,
		actionRegistry,
		MouseShakeDetector,
		type ActionDefinition,
		type ActionTriggerSource
	} from '$lib/actions';

	let {
		title = 'Action island',
		hint = 'Choose an action',
		emptyLabel = 'No actions are available',
		closeLabel = 'Close action island',
		navigationHint = '↑ ↓ navigate · Enter select · Alt+C toggle',
		shakeEnabled = true,
		shakeRespectsReducedMotion = true
	}: {
		title?: string;
		hint?: string;
		emptyLabel?: string;
		closeLabel?: string;
		navigationHint?: string;
		shakeEnabled?: boolean;
		shakeRespectsReducedMotion?: boolean;
	} = $props();

	let overlay: HTMLElement | null = $state(null);
	let scroller: HTMLElement | null = $state(null);
	let selectedIndex = $state(0);
	let runningId = $state<string | null>(null);
	let error = $state('');
	let previousFocus: HTMLElement | null = null;
	let execution: AbortController | null = null;
	const shake = new MouseShakeDetector();

	const actions = $derived(actionRegistry.actions);

	function actionEnabled(action: ActionDefinition): boolean {
		return typeof action.enabled === 'function' ? action.enabled() : action.enabled !== false;
	}

	function actionBadge(action: ActionDefinition): string | undefined {
		return typeof action.badge === 'function' ? action.badge() : action.badge;
	}

	function focusAction(index: number): void {
		if (!actions.length) {
			overlay?.querySelector<HTMLButtonElement>('.close')?.focus();
			return;
		}
		selectedIndex = (index + actions.length) % actions.length;
		requestAnimationFrame(() => {
			overlay
				?.querySelector<HTMLButtonElement>(`[data-action-index="${selectedIndex}"]`)
				?.focus({ preventScroll: true });
			overlay
				?.querySelector<HTMLElement>(`[data-action-index="${selectedIndex}"]`)
				?.scrollIntoView({
					block: 'nearest',
					inline: 'nearest',
					behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'
				});
		});
	}

	function close(): void {
		execution?.abort();
		execution = null;
		runningId = null;
		actionIsland.close();
	}

	async function execute(action: ActionDefinition, source: ActionTriggerSource): Promise<void> {
		if (runningId || !actionEnabled(action)) return;
		error = '';
		execution?.abort();
		execution = new AbortController();
		const controller = execution;
		const dismiss = action.dismiss ?? 'after-run';

		try {
			if (dismiss !== 'immediate') runningId = action.id;
			const task = action.run({ source, signal: controller.signal, close });
			if (dismiss === 'immediate') {
				// Immediate actions continue independently after the visual layer closes.
				execution = null;
				actionIsland.close();
			}
			await task;
			if (!controller.signal.aborted && dismiss === 'after-run') actionIsland.close();
		} catch (cause) {
			if (controller.signal.aborted) return;
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			if (execution === controller) {
				execution = null;
				runningId = null;
			}
		}
	}

	function handleOverlayKeydown(event: KeyboardEvent): void {
		if (event.key === 'Escape') {
			event.preventDefault();
			close();
			return;
		}

		const target = event.target as HTMLElement;
		const indexText = target.dataset.actionIndex;
		const current = indexText === undefined ? selectedIndex : Number(indexText);
		let next: number | null = null;
		if (event.key === 'ArrowRight' || event.key === 'ArrowDown') next = current + 1;
		if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') next = current - 1;
		if (event.key === 'Home') next = 0;
		if (event.key === 'End') next = actions.length - 1;
		if (event.key === 'PageDown') next = current + 4;
		if (event.key === 'PageUp') next = current - 4;
		if (next !== null) {
			event.preventDefault();
			focusAction(Math.max(0, Math.min(actions.length - 1, next)));
		}

		if (event.key === 'Tab') {
			const focusable = [
				...(overlay?.querySelectorAll<HTMLElement>(
					'button:not([disabled]), [href], [tabindex]:not([tabindex="-1"])'
				) ?? [])
			];
			if (!focusable.length) return;
			const first = focusable[0];
			const last = focusable[focusable.length - 1];
			if (event.shiftKey && document.activeElement === first) {
				event.preventDefault();
				last.focus();
			} else if (!event.shiftKey && document.activeElement === last) {
				event.preventDefault();
				first.focus();
			}
		}
	}

	$effect(() => {
		const isOpen = actionIsland.isOpen;
		if (!isOpen) return;
		const previousOverflow = document.documentElement.style.overflow;
		untrack(() => {
			previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
			document.documentElement.style.overflow = 'hidden';
			selectedIndex = Math.min(selectedIndex, Math.max(actions.length - 1, 0));
			error = '';
			requestAnimationFrame(() => focusAction(selectedIndex));
		});

		return () => {
			document.documentElement.style.overflow = previousOverflow;
			execution?.abort();
			execution = null;
			runningId = null;
			previousFocus?.focus({ preventScroll: true });
			previousFocus = null;
		};
	});

	onMount(() => {
		const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');

		function handleGlobalKeydown(event: KeyboardEvent): void {
			if (
				event.altKey &&
				!event.ctrlKey &&
				!event.metaKey &&
				!event.shiftKey &&
				event.code === 'KeyC' &&
				!event.repeat
			) {
				event.preventDefault();
				actionIsland.toggle('hotkey');
			}
		}

		function handlePointerMove(event: PointerEvent): void {
			if (
				!shakeEnabled ||
				event.pointerType === 'touch' ||
				(shakeRespectsReducedMotion && reducedMotion.matches)
			) {
				return;
			}
			if (shake.push(event.clientX, event.clientY, event.timeStamp)) {
				actionIsland.open('shake');
			}
		}

		window.addEventListener('keydown', handleGlobalKeydown);
		window.addEventListener('pointermove', handlePointerMove, { passive: true });
		return () => {
			window.removeEventListener('keydown', handleGlobalKeydown);
			window.removeEventListener('pointermove', handlePointerMove);
		};
	});
</script>

{#if actionIsland.isOpen}
	<div
		bind:this={overlay}
		class="island-overlay"
		role="dialog"
		tabindex="-1"
		aria-modal="true"
		aria-labelledby="action-island-title"
		onkeydown={handleOverlayKeydown}
		onclick={(event) => {
			if (event.target === event.currentTarget) close();
		}}
	>
		<div class="light light-one"></div>
		<div class="light light-two"></div>

		<header class="island-header">
			<div>
				<p class="eyebrow">{hint}</p>
				<h2 id="action-island-title">{title}</h2>
			</div>
			<button class="close" type="button" aria-label={closeLabel} onclick={close}>
				<span aria-hidden="true">×</span>
				<kbd>Esc</kbd>
			</button>
		</header>

		<div bind:this={scroller} class="action-scroller">
			{#if actions.length}
				<div class="action-grid" aria-busy={runningId !== null}>
					{#each actions as action, index (action.id)}
						{@const badge = actionBadge(action)}
						<button
							class="action tone-{action.tone ?? 'neutral'}"
							class:selected={selectedIndex === index}
							class:running={runningId === action.id}
							type="button"
							data-action-index={index}
							disabled={!actionEnabled(action) || (runningId !== null && runningId !== action.id)}
							style:--action-delay={`${Math.min(index * 34, 306)}ms`}
							onfocus={() => (selectedIndex = index)}
							onpointerenter={() => (selectedIndex = index)}
							onclick={() => execute(action, 'pointer')}
							onkeydown={(event) => {
								if (event.key === 'Enter' || event.key === ' ') {
									event.preventDefault();
									execute(action, 'keyboard');
								}
							}}
						>
							<span class="action-top">
								<span class="action-icon" aria-hidden="true">{action.icon ?? '✦'}</span>
								{#if badge}<span class="badge">{badge}</span>{/if}
								{#if runningId === action.id}<span class="spinner" aria-hidden="true"></span>{/if}
							</span>
							<span class="action-copy">
								<strong>{action.label}</strong>
								{#if action.description}<span>{action.description}</span>{/if}
							</span>
							{#if action.group}<span class="group">{action.group}</span>{/if}
						</button>
					{/each}
				</div>
			{:else}
				<p class="empty">{emptyLabel}</p>
			{/if}
		</div>

		<footer class="island-footer">
			<p>{navigationHint}</p>
			{#if error}<p class="error" role="alert">{error}</p>{/if}
		</footer>
	</div>
{/if}

<style>
	.island-overlay {
		position: fixed;
		inset: 0;
		z-index: 2147483000;
		isolation: isolate;
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		padding: clamp(1rem, 3.2vw, 3rem);
		overflow: hidden;
		color: #f5f9ff;
		background:
			linear-gradient(145deg, rgba(8, 15, 24, 0.74), rgba(5, 10, 16, 0.9)),
			rgba(7, 12, 18, 0.76);
		backdrop-filter: blur(28px) saturate(145%);
		-webkit-backdrop-filter: blur(28px) saturate(145%);
		animation: reveal 220ms cubic-bezier(0.16, 1, 0.3, 1) both;
	}

	.island-overlay::before {
		content: '';
		position: absolute;
		z-index: -1;
		inset: 0;
		pointer-events: none;
		background-image:
			linear-gradient(rgba(255, 255, 255, 0.022) 1px, transparent 1px),
			linear-gradient(90deg, rgba(255, 255, 255, 0.022) 1px, transparent 1px);
		background-size: 42px 42px;
		mask-image: radial-gradient(circle at center, black, transparent 78%);
	}

	.light {
		position: absolute;
		z-index: -2;
		width: min(70vw, 900px);
		aspect-ratio: 1;
		border-radius: 50%;
		filter: blur(70px);
		opacity: 0.3;
		pointer-events: none;
	}

	.light-one {
		top: -52%;
		left: -12%;
		background: #3aa9ff;
	}

	.light-two {
		right: -24%;
		bottom: -70%;
		background: #4fffc1;
	}

	.island-header,
	.island-footer {
		position: relative;
		z-index: 1;
		width: min(1180px, 100%);
		margin-inline: auto;
	}

	.island-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1.25rem;
		padding-bottom: clamp(1.25rem, 3vh, 2rem);
	}

	.eyebrow {
		margin: 0 0 0.35rem;
		color: rgba(206, 222, 237, 0.66);
		font-size: 0.72rem;
		font-weight: 600;
		letter-spacing: 0.15em;
		text-transform: uppercase;
	}

	h2 {
		margin: 0;
		font-family: var(--font-display, system-ui, sans-serif);
		font-size: clamp(2rem, 4.6vw, 4.4rem);
		font-weight: 420;
		line-height: 0.98;
		letter-spacing: -0.04em;
		text-shadow: 0 12px 40px rgba(82, 180, 255, 0.15);
	}

	button {
		font: inherit;
	}

	.close {
		display: inline-flex;
		align-items: center;
		gap: 0.65rem;
		min-height: 48px;
		padding: 0.45rem 0.65rem 0.45rem 1rem;
		border: 1px solid rgba(255, 255, 255, 0.14);
		border-radius: 999px;
		color: rgba(240, 247, 255, 0.84);
		background: linear-gradient(180deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.035));
		box-shadow:
			inset 0 1px rgba(255, 255, 255, 0.14),
			0 12px 34px rgba(0, 0, 0, 0.22);
		cursor: pointer;
	}

	.close span {
		font-size: 1.5rem;
		font-weight: 300;
		line-height: 1;
	}

	kbd {
		padding: 0.28rem 0.5rem;
		border: 1px solid rgba(255, 255, 255, 0.11);
		border-radius: 999px;
		color: rgba(215, 229, 242, 0.62);
		background: rgba(0, 0, 0, 0.18);
		font-family: inherit;
		font-size: 0.68rem;
	}

	.action-scroller {
		width: min(1220px, calc(100% + 1rem));
		margin-inline: auto;
		padding: 0.5rem;
		overflow: auto;
		overscroll-behavior: contain;
		scrollbar-width: thin;
		scrollbar-color: rgba(131, 197, 255, 0.35) transparent;
		mask-image: linear-gradient(transparent, black 1rem, black calc(100% - 1rem), transparent);
	}

	.action-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 250px), 1fr));
		gap: clamp(0.8rem, 1.5vw, 1.25rem);
		padding-block: 1.25rem;
	}

	.action {
		--tone: 106, 190, 255;
		position: relative;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		min-height: clamp(176px, 24vh, 260px);
		padding: clamp(1.15rem, 2.2vw, 1.75rem);
		overflow: hidden;
		border: 1px solid rgba(255, 255, 255, 0.14);
		border-radius: clamp(22px, 2.4vw, 32px);
		color: #f5f9ff;
		text-align: left;
		background:
			linear-gradient(145deg, rgba(var(--tone), 0.13), transparent 52%),
			linear-gradient(155deg, rgba(255, 255, 255, 0.12), rgba(255, 255, 255, 0.035));
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.19),
			inset 0 -1px 0 rgba(0, 0, 0, 0.22),
			0 22px 55px rgba(0, 0, 0, 0.28),
			0 2px 8px rgba(0, 0, 0, 0.25);
		backdrop-filter: blur(18px) saturate(130%);
		-webkit-backdrop-filter: blur(18px) saturate(130%);
		cursor: pointer;
		transform: translateY(0);
		transition:
			transform 180ms cubic-bezier(0.16, 1, 0.3, 1),
			border-color 180ms ease,
			box-shadow 180ms ease,
			background 180ms ease;
		animation: action-in 420ms cubic-bezier(0.16, 1, 0.3, 1) both;
		animation-delay: var(--action-delay);
	}

	.action::after {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
		background: linear-gradient(125deg, rgba(255, 255, 255, 0.11), transparent 30%);
		pointer-events: none;
	}

	.action:hover,
	.action.selected {
		border-color: rgba(var(--tone), 0.64);
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.24),
			inset 0 -1px 0 rgba(0, 0, 0, 0.2),
			0 25px 65px rgba(0, 0, 0, 0.34),
			0 0 0 4px rgba(var(--tone), 0.08),
			0 0 50px rgba(var(--tone), 0.1);
		transform: translateY(-5px);
	}

	.action:focus-visible {
		outline: 2px solid rgba(var(--tone), 0.9);
		outline-offset: 4px;
	}

	.action:active {
		transform: translateY(-1px) scale(0.985);
		box-shadow:
			inset 0 3px 8px rgba(0, 0, 0, 0.2),
			0 12px 32px rgba(0, 0, 0, 0.3);
	}

	.action:disabled {
		cursor: not-allowed;
		filter: saturate(0.45);
		opacity: 0.48;
		transform: none;
	}

	.action.tone-accent {
		--tone: 70, 180, 255;
	}

	.action.tone-success {
		--tone: 72, 239, 184;
	}

	.action.tone-warning {
		--tone: 255, 191, 84;
	}

	.action.tone-danger {
		--tone: 255, 101, 122;
	}

	.action-top {
		display: flex;
		align-items: flex-start;
		gap: 0.65rem;
	}

	.action-icon {
		display: grid;
		width: 58px;
		height: 58px;
		place-items: center;
		flex: 0 0 auto;
		border: 1px solid rgba(255, 255, 255, 0.15);
		border-radius: 18px;
		background:
			linear-gradient(145deg, rgba(var(--tone), 0.22), rgba(255, 255, 255, 0.045)),
			rgba(0, 0, 0, 0.08);
		box-shadow:
			inset 0 1px rgba(255, 255, 255, 0.2),
			0 10px 25px rgba(0, 0, 0, 0.2);
		font-size: 1.6rem;
	}

	.badge {
		max-width: 10rem;
		padding: 0.35rem 0.6rem;
		overflow: hidden;
		border: 1px solid rgba(var(--tone), 0.24);
		border-radius: 999px;
		color: rgb(var(--tone));
		background: rgba(var(--tone), 0.09);
		font-size: 0.68rem;
		font-weight: 650;
		letter-spacing: 0.04em;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.spinner {
		width: 22px;
		height: 22px;
		margin-left: auto;
		border: 2px solid rgba(255, 255, 255, 0.16);
		border-top-color: rgb(var(--tone));
		border-radius: 50%;
		animation: spin 700ms linear infinite;
	}

	.action-copy {
		display: grid;
		gap: 0.45rem;
		margin-top: 1.5rem;
	}

	.action-copy strong {
		font-family: var(--font-display, system-ui, sans-serif);
		font-size: clamp(1.18rem, 1.8vw, 1.55rem);
		font-weight: 540;
		line-height: 1.08;
		letter-spacing: -0.025em;
	}

	.action-copy > span {
		display: -webkit-box;
		overflow: hidden;
		color: rgba(217, 230, 242, 0.62);
		font-size: 0.86rem;
		line-height: 1.45;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
	}

	.group {
		align-self: flex-start;
		margin-top: 1.1rem;
		color: rgba(195, 214, 230, 0.42);
		font-size: 0.65rem;
		font-weight: 620;
		letter-spacing: 0.12em;
		text-transform: uppercase;
	}

	.empty {
		display: grid;
		min-height: 40vh;
		margin: 0;
		place-items: center;
		color: rgba(210, 225, 238, 0.52);
		font-size: clamp(1rem, 2vw, 1.3rem);
	}

	.island-footer {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 1rem;
		padding-top: clamp(0.8rem, 2.2vh, 1.5rem);
		color: rgba(198, 216, 232, 0.44);
		font-size: 0.7rem;
		letter-spacing: 0.04em;
	}

	.island-footer p {
		margin: 0;
	}

	.island-footer .error {
		max-width: min(60ch, 55vw);
		color: #ff9eac;
		text-align: right;
	}

	@keyframes reveal {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	@keyframes action-in {
		from {
			opacity: 0;
			transform: translateY(24px) scale(0.97);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	@media (max-width: 620px) {
		.island-overlay {
			padding: 1rem;
		}

		.island-header {
			align-items: flex-start;
		}

		.close kbd,
		.island-footer > p:first-child {
			display: none;
		}

		.action-grid {
			grid-template-columns: 1fr;
		}

		.action {
			min-height: 170px;
		}

		.island-footer .error {
			max-width: 100%;
			text-align: left;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.island-overlay,
		.action,
		.spinner {
			animation: none;
		}

		.action,
		.close {
			scroll-behavior: auto;
			transition-duration: 0.01ms;
		}

		.action:hover,
		.action.selected {
			transform: none;
		}
	}

	@media (prefers-contrast: more) {
		.island-overlay {
			background: rgba(3, 7, 11, 0.96);
		}

		.action,
		.close {
			border-color: rgba(255, 255, 255, 0.5);
		}
	}
</style>
