<script module lang="ts">
	export type WindowChromeAction = 'minimize' | 'toggleMaximize' | 'close';

	export interface WindowChromeError {
		action: WindowChromeAction;
		error: unknown;
	}

	export interface WindowChromeLabels {
		titlebar: string;
		controls: string;
		minimize: string;
		maximize: string;
		restore: string;
		close: string;
	}

	const DEFAULT_LABELS: WindowChromeLabels = {
		titlebar: 'Application title bar',
		controls: 'Window controls',
		minimize: 'Minimize window',
		maximize: 'Maximize window',
		restore: 'Restore window',
		close: 'Close window'
	};
</script>

<script lang="ts">
	import { onMount } from 'svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import type { UnlistenFn } from '@tauri-apps/api/event';

	let {
		title = 'EXSUL',
		labels = {},
		onactionerror
	}: {
		title?: string;
		labels?: Partial<WindowChromeLabels>;
		onactionerror?: (event: WindowChromeError) => void;
	} = $props();

	let nativeWindow = $state<ReturnType<typeof getCurrentWindow>>();
	let native = $state(false);
	let maximized = $state(false);
	let focused = $state(true);

	const text = $derived({ ...DEFAULT_LABELS, ...labels });

	async function syncMaximized() {
		if (!nativeWindow) return;

		try {
			maximized = await nativeWindow.isMaximized();
		} catch {
			// A stale icon is preferable to surfacing an IPC failure in the title bar.
		}
	}

	async function run(action: WindowChromeAction) {
		// Vite's browser preview deliberately keeps the chrome visual but inert.
		if (!nativeWindow) return;

		try {
			if (action === 'minimize') {
				await nativeWindow.minimize();
				return;
			}

			if (action === 'toggleMaximize') {
				await nativeWindow.toggleMaximize();
				await syncMaximized();
				return;
			}

			await nativeWindow.close();
		} catch (error) {
			onactionerror?.({ action, error });
		}
	}

	function stopControlPointer(event: PointerEvent) {
		event.stopPropagation();
	}

	function handleTitlebarDoubleClick(event: MouseEvent) {
		if ((event.target as HTMLElement).closest('button')) return;
		void run('toggleMaximize');
	}

	onMount(() => {
		native = isTauri();
		if (!native) return;

		nativeWindow = getCurrentWindow();
		let cancelled = false;
		const unlisteners: UnlistenFn[] = [];

		const keep = async (listener: Promise<UnlistenFn>) => {
			try {
				const unlisten = await listener;
				if (cancelled) unlisten();
				else unlisteners.push(unlisten);
			} catch {
				// The controls remain usable even if a platform event is unavailable.
			}
		};

		void syncMaximized();
		void keep(nativeWindow.onResized(() => void syncMaximized()));
		void keep(
			nativeWindow.onFocusChanged(({ payload }) => {
				focused = payload;
			})
		);

		return () => {
			cancelled = true;
			for (const unlisten of unlisteners) unlisten();
			nativeWindow = undefined;
		};
	});
</script>

<div
	class="chrome-shell"
	class:inactive={!focused}
	class:web-fallback={!native}
	data-native={native}
>
	<div
		class="chrome"
		data-tauri-drag-region
		role="banner"
		aria-label={text.titlebar}
		ondblclick={handleTitlebarDoubleClick}
	>
		<div class="brand" aria-hidden="true">
			<span class="brand-mark">
				<span class="brand-core"></span>
			</span>
			<span class="wordmark">{title}</span>
		</div>

		<div class="drag-cue" aria-hidden="true">
			<span></span>
		</div>

		<div class="window-controls" role="group" aria-label={text.controls}>
			<button
				class="window-control minimize"
				type="button"
				aria-label={text.minimize}
				aria-disabled={!native}
				title={text.minimize}
				onpointerdown={stopControlPointer}
				onclick={() => void run('minimize')}
			>
				<svg viewBox="0 0 16 16" aria-hidden="true">
					<path d="M3.25 8.75h9.5" />
				</svg>
			</button>

			<button
				class="window-control maximize"
				type="button"
				aria-label={maximized ? text.restore : text.maximize}
				aria-disabled={!native}
				aria-pressed={maximized}
				title={maximized ? text.restore : text.maximize}
				onpointerdown={stopControlPointer}
				onclick={() => void run('toggleMaximize')}
			>
				{#if maximized}
					<svg class="restore-icon" viewBox="0 0 16 16" aria-hidden="true">
						<path d="M5.25 5.25v-1.5h7v7h-1.5" />
						<rect x="3.75" y="5.75" width="6.5" height="6.5" rx="1" />
					</svg>
				{:else}
					<svg viewBox="0 0 16 16" aria-hidden="true">
						<rect x="3.5" y="3.5" width="9" height="9" rx="1.35" />
					</svg>
				{/if}
			</button>

			<button
				class="window-control close"
				type="button"
				aria-label={text.close}
				aria-disabled={!native}
				title={text.close}
				onpointerdown={stopControlPointer}
				onclick={() => void run('close')}
			>
				<svg viewBox="0 0 16 16" aria-hidden="true">
					<path d="m4.25 4.25 7.5 7.5m0-7.5-7.5 7.5" />
				</svg>
			</button>
		</div>
	</div>
</div>

<style>
	.chrome-shell {
		--chrome-height: 48px;
		--chrome-text: rgba(245, 249, 250, 0.9);
		--chrome-muted: rgba(218, 229, 231, 0.58);
		--chrome-line: rgba(224, 241, 242, 0.14);
		--chrome-highlight: rgba(255, 255, 255, 0.16);
		--chrome-surface: rgba(36, 45, 47, 0.7);
		--chrome-surface-deep: rgba(17, 23, 25, 0.82);
		--chrome-focus: rgba(174, 184, 173, 0.76);
		position: fixed;
		z-index: 10000;
		inset: 0 0 auto;
		height: calc(var(--chrome-height) + 12px);
		padding: 8px 10px 4px;
		box-sizing: border-box;
		pointer-events: none;
		user-select: none;
		-webkit-user-select: none;
	}

	.chrome {
		position: relative;
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
		align-items: center;
		width: 100%;
		height: var(--chrome-height);
		box-sizing: border-box;
		overflow: hidden;
		border: 1px solid var(--chrome-line);
		border-radius: 15px;
		background:
			linear-gradient(105deg, rgba(157, 197, 197, 0.08), transparent 34%),
			linear-gradient(180deg, var(--chrome-surface), var(--chrome-surface-deep));
		box-shadow:
			inset 0 1px 0 var(--chrome-highlight),
			inset 0 -1px 0 rgba(0, 0, 0, 0.3),
			0 10px 30px rgba(0, 0, 0, 0.24),
			0 2px 6px rgba(0, 0, 0, 0.22);
		backdrop-filter: blur(24px) saturate(145%);
		-webkit-backdrop-filter: blur(24px) saturate(145%);
		pointer-events: auto;
		transition:
			border-color 180ms ease,
			filter 180ms ease,
			box-shadow 180ms ease;
	}

	.chrome::before {
		content: '';
		position: absolute;
		inset: 0;
		background:
			radial-gradient(circle at 16% -100%, rgba(164, 238, 226, 0.17), transparent 47%),
			linear-gradient(90deg, transparent 42%, rgba(255, 255, 255, 0.025) 58%, transparent 72%);
		pointer-events: none;
	}

	.brand {
		position: relative;
		z-index: 1;
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
		padding-left: 14px;
		pointer-events: none;
	}

	.brand-mark {
		position: relative;
		display: grid;
		width: 20px;
		height: 20px;
		flex: 0 0 auto;
		place-items: center;
		border: 1px solid rgba(218, 247, 243, 0.22);
		border-radius: 7px;
		background:
			linear-gradient(145deg, rgba(230, 255, 251, 0.16), rgba(70, 111, 109, 0.1)),
			rgba(10, 16, 18, 0.32);
		box-shadow:
			inset 0 1px 1px rgba(255, 255, 255, 0.18),
			0 4px 12px rgba(0, 0, 0, 0.24);
		transform: rotate(45deg);
	}

	.brand-core {
		width: 5px;
		height: 5px;
		border-radius: 2px;
		background: rgba(188, 249, 238, 0.94);
		box-shadow:
			0 0 8px rgba(116, 235, 218, 0.66),
			0 0 18px rgba(116, 235, 218, 0.24);
	}

	.wordmark {
		overflow: hidden;
		color: var(--chrome-text);
		font-family: inherit;
		font-size: 0.69rem;
		font-weight: 610;
		line-height: 1;
		letter-spacing: 0.24em;
		text-overflow: ellipsis;
		white-space: nowrap;
		text-shadow: 0 1px 8px rgba(170, 233, 225, 0.14);
	}

	.drag-cue {
		position: relative;
		z-index: 1;
		display: grid;
		width: 48px;
		height: 24px;
		place-items: center;
		pointer-events: none;
	}

	.drag-cue span {
		width: 24px;
		height: 3px;
		border-radius: 999px;
		background: linear-gradient(90deg, transparent, var(--chrome-muted), transparent);
		box-shadow: 0 1px 0 rgba(0, 0, 0, 0.42);
		opacity: 0.54;
	}

	.window-controls {
		position: relative;
		z-index: 2;
		justify-self: end;
		display: flex;
		align-items: center;
		gap: 5px;
		padding-right: 7px;
	}

	.window-control {
		position: relative;
		display: grid;
		width: 40px;
		height: 34px;
		margin: 0;
		padding: 0;
		place-items: center;
		border: 1px solid rgba(230, 246, 246, 0.1);
		border-radius: 10px;
		outline: none;
		background:
			linear-gradient(180deg, rgba(255, 255, 255, 0.075), rgba(255, 255, 255, 0.018)),
			rgba(10, 15, 17, 0.18);
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.09),
			0 2px 6px rgba(0, 0, 0, 0.15);
		color: var(--chrome-muted);
		cursor: pointer;
		transition:
			color 140ms ease,
			border-color 140ms ease,
			background 140ms ease,
			box-shadow 140ms ease,
			transform 100ms ease;
		-webkit-tap-highlight-color: transparent;
	}

	.window-control svg {
		width: 16px;
		height: 16px;
		overflow: visible;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.35;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.window-control:hover {
		border-color: rgba(223, 247, 244, 0.2);
		background:
			linear-gradient(180deg, rgba(255, 255, 255, 0.13), rgba(255, 255, 255, 0.035)),
			rgba(30, 48, 50, 0.34);
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.15),
			0 5px 12px rgba(0, 0, 0, 0.22);
		color: rgba(250, 255, 255, 0.96);
	}

	.window-control:active {
		transform: translateY(1px) scale(0.97);
		box-shadow:
			inset 0 1px 3px rgba(0, 0, 0, 0.24),
			0 1px 2px rgba(0, 0, 0, 0.16);
	}

	.window-control:focus-visible {
		border-color: var(--chrome-focus);
		box-shadow:
			0 0 0 2px rgba(14, 22, 23, 0.92),
			0 0 0 4px rgba(174, 184, 173, 0.42);
		color: rgba(250, 255, 255, 0.98);
	}

	.window-control.close {
		color: rgba(241, 207, 204, 0.7);
	}

	.window-control.close:hover {
		border-color: rgba(255, 147, 137, 0.34);
		background:
			linear-gradient(180deg, rgba(255, 177, 168, 0.19), rgba(186, 58, 52, 0.12)),
			rgba(83, 27, 27, 0.34);
		box-shadow:
			inset 0 1px 0 rgba(255, 221, 217, 0.16),
			0 5px 15px rgba(72, 17, 17, 0.27);
		color: rgba(255, 235, 232, 0.98);
	}

	.inactive .chrome {
		border-color: rgba(224, 241, 242, 0.08);
		filter: saturate(0.72);
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.09),
			0 7px 24px rgba(0, 0, 0, 0.2);
	}

	.inactive .brand-core {
		opacity: 0.58;
	}

	.web-fallback .chrome {
		cursor: default;
	}

	.web-fallback .window-control {
		cursor: not-allowed;
	}

	@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px))) {
		.chrome {
			background: linear-gradient(180deg, rgb(44, 54, 56), rgb(20, 27, 29));
		}
	}

	@media (max-width: 560px) {
		.chrome-shell {
			padding-inline: 6px;
		}

		.chrome {
			grid-template-columns: 44px 1fr auto;
		}

		.wordmark {
			display: none;
		}

		.drag-cue {
			justify-self: center;
		}

		.window-control {
			width: 38px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.chrome,
		.window-control {
			transition: none;
		}
	}

	@media (forced-colors: active) {
		.chrome,
		.window-control {
			border: 1px solid ButtonText;
			background: Canvas;
			color: ButtonText;
			box-shadow: none;
		}

		.window-control:focus-visible {
			outline: 2px solid Highlight;
			outline-offset: -3px;
		}
	}
</style>
