<script lang="ts">
	// Layer 3 — сцена результатов. Не отдельное окно: карточки проявляются в
	// пространстве, AI-резюме течёт как часть единого контекстного потока.
	import { t } from '$lib/i18n';
	import ProductCard from './ProductCard.svelte';
	import type { AssistantResponse } from '$lib/types';

	let { response }: { response: AssistantResponse } = $props();

	const results = $derived(response.search.results);
	const count = $derived(results.length);
	const summary = $derived(response.answer?.text || response.search.assistant_summary);
</script>

<div class="scene">
	<!-- Контекстный слой: ответ ассистента + счётчик, типографикой, не коробкой -->
	<div class="stream">
		<span class="count">{t('results.found', { count })}</span>
		{#if summary}
			<p class="summary">{summary}</p>
		{/if}
		{#if response.answer_error}
			<p class="soft-error">{t('assistant.errorFallback')}</p>
		{/if}
	</div>

	{#if count === 0}
		<div class="empty">
			<p class="empty-title">{t('results.empty')}</p>
			<p class="empty-hint">{t('results.emptyHint')}</p>
		</div>
	{:else}
		<div class="grid">
			{#each results as r, i (r.item_id)}
				<ProductCard result={r} index={i} />
			{/each}
		</div>
	{/if}
</div>

<style>
	.scene {
		display: flex;
		flex-direction: column;
		gap: 1.8rem;
		width: min(1240px, 94vw);
		margin: 0 auto;
	}
	.stream {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.7rem;
		text-align: center;
		animation: rise var(--dur) var(--ease-out) both;
	}
	.count {
		font-family: var(--font-display);
		font-size: 0.72rem;
		font-weight: 650;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--accent);
	}
	.summary {
		font-family: var(--font-display);
		font-weight: 590;
		font-size: clamp(1.65rem, 3.5vw, 3.25rem);
		line-height: 1.13;
		letter-spacing: 0.025em;
		text-transform: uppercase;
		text-wrap: balance;
		color: var(--text-strong);
		max-width: 32ch;
		text-shadow: 0 8px 44px rgba(122, 234, 220, 0.12);
	}
	.soft-error {
		font-size: 0.78rem;
		color: var(--text-soft);
		opacity: 0.8;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(270px, 1fr));
		gap: 1.15rem;
	}
	.empty {
		padding: 3rem 0;
		text-align: center;
	}
	.empty-title {
		font-family: var(--font-display);
		font-size: clamp(1.6rem, 3vw, 2.5rem);
		font-weight: 620;
		text-transform: uppercase;
		color: var(--text-strong);
	}
	.empty-hint {
		margin-top: 0.4rem;
		color: var(--text-faint);
	}
	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(18px);
			filter: blur(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
			filter: blur(0);
		}
	}
</style>
