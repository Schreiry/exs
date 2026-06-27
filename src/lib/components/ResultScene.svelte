<script lang="ts">
	// Layer 3 — сцена результатов. Не отдельное окно: карточки проявляются в
	// пространстве, AI-резюме течёт как часть единого контекстного потока.
	import { t } from '$lib/i18n';
	import ProductCard from './ProductCard.svelte';
	import { getItemImage } from '$lib/tauri/commands';
	import type { AssistantResponse } from '$lib/types';

	let { response }: { response: AssistantResponse } = $props();

	const results = $derived(response.search.results);
	const count = $derived(results.length);
	const summary = $derived(response.answer?.text || response.search.assistant_summary);

	// Cache of item_id -> data: URL (or undefined = missing). Re-fetches are
	// skipped when the URL is already known.
	let imageCache = $state<Record<string, string | null>>({});

	// The top result is what gets the hero image treatment when there's a
	// single match (the user's main case: search "სათამაშო ჯიპი" → 1 hit).
	const hero = $derived(results[0]);
	const heroUrl = $derived(hero ? imageCache[hero.item_id] : undefined);

	$effect(() => {
		// Re-fetch whenever the result set changes. We only request images for
		// items that have image_path set (saves an IPC roundtrip otherwise).
		for (const r of results) {
			if (!r.item.image_path) continue;
			if (r.item_id in imageCache) continue;
			// Mark as in-flight so a second pass doesn't double-request.
			imageCache[r.item_id] = null;
			getItemImage(r.item_id)
				.then((img) => {
					imageCache[r.item_id] = img
						? `data:${img.mime};base64,${img.base64}`
						: null;
				})
				.catch(() => {
					imageCache[r.item_id] = null;
				});
		}
	});

	// Called by ProductCard after a successful upload. Re-fetch the image bytes
	// and refresh the cache entry; the card re-renders automatically. The hero
	// conditional is driven by `heroUrl`, which is $derived from this cache,
	// so the hero also appears without needing a new search.
	function handleUploaded(itemId: string) {
		getItemImage(itemId)
			.then((img) => {
				imageCache[itemId] = img ? `data:${img.mime};base64,${img.base64}` : null;
			})
			.catch(() => {
				imageCache[itemId] = null;
			});
	}
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
		{#if count === 1 && hero && heroUrl}
			<!-- Hero: single match with an image → featured, centered above the card -->
			<figure class="hero">
				<img src={heroUrl} alt={hero.title} />
				<figcaption class="hero-cap">
					<span class="hero-title">{hero.title}</span>
					<span class="hero-cat">{hero.item.category}</span>
				</figcaption>
			</figure>
		{/if}

		<div class="grid" class:single={count === 1}>
			{#each results as r, i (r.item_id)}
				<ProductCard result={r} index={i} imageUrl={imageCache[r.item_id]} onUploaded={handleUploaded} />
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
	.hero {
		margin: 0 auto;
		max-width: 560px;
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.9rem;
		animation: rise var(--dur-slow) var(--ease-out) both;
	}
	.hero img {
		width: 100%;
		max-height: 360px;
		object-fit: cover;
		border-radius: 20px;
		border: 1px solid var(--glass-border);
		box-shadow:
			var(--glass-shadow),
			0 30px 80px -30px rgba(0, 0, 0, 0.6);
		background: rgba(0, 0, 0, 0.25);
	}
	.hero-cap {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.2rem;
		text-align: center;
	}
	.hero-title {
		font-family: var(--font-display);
		font-weight: 400;
		font-size: clamp(1.4rem, 2.6vw, 1.9rem);
		color: var(--text-strong);
		letter-spacing: 0.01em;
	}
	.hero-cat {
		font-size: 0.78rem;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
	.stream {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 0.7rem;
		text-align: left;
		animation: rise var(--dur) var(--ease-out) both;
		width: 100%;
	}
	.count {
		font-family: var(--font-ui);
		font-size: 0.72rem;
		font-weight: 650;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--accent);
	}
	.summary {
		/* Use UI font (Noto Sans Georgian + Jost) — Jost display has no
		   Mkhedruli glyphs and falls back to a cramped system font. */
		font-family: var(--font-ui);
		font-weight: 400;
		font-size: clamp(1rem, 1.55vw, 1.18rem);
		line-height: 1.65;
		letter-spacing: 0.005em;
		color: var(--text-strong);
		max-width: 70ch;
		text-align: left;
		align-self: flex-start;
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
	.grid.single {
		grid-template-columns: minmax(280px, 420px);
		justify-content: center;
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
