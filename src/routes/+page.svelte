<script lang="ts">
	// Главный экран — «пустое контекстное пространство». Один <main> = вся сцена.
	// Состояния (idle/searching/results/error) — это режимы одной сцены, не окна.
	import VoidBackground from '$lib/components/VoidBackground.svelte';
	import ContextInput from '$lib/components/ContextInput.svelte';
	import ResultScene from '$lib/components/ResultScene.svelte';
	import { scene } from '$lib/scene/scene.svelte';
	import { t } from '$lib/i18n';
	import { isTauri, seedDemoItems, aiGetStatus } from '$lib/tauri/commands';

	let draft = $state('');
	let provider = $state<string | null>(null);

	function submit(q: string) {
		const query = q.trim();
		if (!query) return;
		draft = '';
		scene.submit(query);
	}

	// Pointer parallax → CSS vars (read by VoidBackground & gradient).
	function onMove(e: PointerEvent) {
		const nx = (e.clientX / window.innerWidth) * 2 - 1;
		const ny = (e.clientY / window.innerHeight) * 2 - 1;
		const root = document.documentElement.style;
		root.setProperty('--px', nx.toFixed(3));
		root.setProperty('--py', ny.toFixed(3));
	}

	// First-run conveniences (Tauri only): seed demo data + read provider status.
	$effect(() => {
		if (!isTauri()) {
			provider = 'mock';
			return;
		}
		seedDemoItems().catch(() => {});
		aiGetStatus()
			.then((s) => (provider = s.selected))
			.catch(() => {});
	});
</script>

<svelte:window onpointermove={onMove} />

<VoidBackground />

<main class="stage mode-{scene.mode}">
	<header class="brand" class:dim={scene.mode === 'results'}>
		<span class="logo">{t('app.title')}</span>
		<span class="tagline">{t('app.tagline')}</span>
	</header>

	<section class="console">
		<ContextInput bind:value={draft} compact={scene.mode !== 'idle'} onsubmit={submit} />

		{#if scene.mode === 'searching'}
			<div class="status pulse">{t('state.thinking')}</div>
		{:else if scene.mode === 'error'}
			<div class="status err">{t('state.error')} — {scene.error}</div>
		{/if}
	</section>

	{#if scene.mode === 'results' && scene.response}
		<div class="results-wrap">
			<ResultScene response={scene.response} />
		</div>
	{/if}

	{#if provider}
		<footer class="provider">AI: {provider}</footer>
	{/if}
</main>

<style>
	.stage {
		position: relative;
		z-index: 1;
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 2rem 1.5rem 3rem;
		transition: justify-content var(--dur) var(--ease-out);
	}
	/* idle / searching: everything centered in the void */
	.mode-idle,
	.mode-searching {
		justify-content: center;
	}
	.mode-results {
		justify-content: flex-start;
		overflow-y: auto;
	}

	.brand {
		text-align: center;
		margin-bottom: 2.2rem;
		transition: opacity var(--dur) var(--ease-soft), transform var(--dur) var(--ease-out);
		animation: wake var(--dur-slow) var(--ease-out) both;
	}
	.brand.dim {
		opacity: 0;
		transform: translateY(-12px);
		height: 0;
		margin: 0;
		pointer-events: none;
	}
	.logo {
		display: block;
		font-family: var(--font-display);
		font-weight: 200;
		font-size: clamp(2.4rem, 7vw, 4rem);
		letter-spacing: 0.16em;
		color: var(--text-strong);
		text-shadow: 0 2px 30px var(--accent-glow);
	}
	.tagline {
		display: block;
		margin-top: 0.5rem;
		font-size: 0.95rem;
		color: var(--text-soft);
		letter-spacing: 0.04em;
	}

	.console {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
	}
	.mode-results .console {
		max-width: 1100px;
		align-items: stretch;
		margin-bottom: 2rem;
	}

	.status {
		font-size: 0.9rem;
		color: var(--text-soft);
		letter-spacing: 0.03em;
	}
	.status.err {
		color: var(--danger);
		max-width: 60ch;
		text-align: center;
	}
	.pulse {
		animation: pulse 1.4s ease-in-out infinite;
	}

	.results-wrap {
		width: 100%;
		animation: rise var(--dur) var(--ease-out) both;
	}

	.provider {
		position: fixed;
		left: 1rem;
		bottom: 0.8rem;
		font-size: 0.72rem;
		color: var(--text-faint);
		letter-spacing: 0.05em;
		z-index: 2;
		opacity: 0.6;
	}

	@keyframes wake {
		from {
			opacity: 0;
			transform: translateY(10px) scale(0.99);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(14px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
	@keyframes pulse {
		0%,
		100% {
			opacity: 0.45;
		}
		50% {
			opacity: 1;
		}
	}
</style>
