<script lang="ts">
	// Layer 3 — product card. Появляется из глубины (не из модалки), со
	// стаггером по индексу. Стиль варьируется по card_style_hint.
	import { t } from '$lib/i18n';
	import { isTauri } from '$lib/tauri/commands';
	import { appAssetUrl } from '$lib/tauri/assets';
	import type { ProductSearchResult } from '$lib/types';

	let { result, index = 0 }: { result: ProductSearchResult; index?: number } = $props();

	const item = $derived(result.item);
	const pct = $derived(Math.round(result.confidence * 100));
	const delay = $derived(Math.min(index * 55, 600));
	let imageUrl = $state<string | null>(null);

	$effect(() => {
		const relative = item.image_path;
		if (!relative || !isTauri()) {
			imageUrl = null;
			return;
		}
		let disposed = false;
		appAssetUrl(relative)
			.then((url) => {
				if (!disposed) imageUrl = url;
			})
			.catch(() => {
				if (!disposed) imageUrl = null;
			});
		return () => {
			disposed = true;
		};
	});
</script>

<article
	class="card hint-{result.card_style_hint}"
	style="--delay: {delay}ms; --tint: {item.card_color || 'transparent'}"
>
	{#if imageUrl}
		<div class="media">
			<img src={imageUrl} alt={result.title} loading="lazy" decoding="async" />
			<div class="media-shade"></div>
		</div>
	{/if}
	<header>
		<h3>{result.title}</h3>
		<span class="cat">{item.category}</span>
	</header>

	<div class="metrics">
		<div class="metric">
			<span class="label">{t('card.price')}</span>
			<span class="value">{item.current_price.toFixed(2)} {t('units.currency')}</span>
		</div>
		<div class="metric">
			<span class="label">{t('card.stock')}</span>
			<span class="value" class:low={item.current_stock <= 3}>{item.current_stock}</span>
		</div>
	</div>

	<footer>
		<span class="reason">{result.reason}</span>
		<span class="conf" title={t('card.confidence')}>
			<span class="bar" style="--w: {pct}%"></span>
			<span class="pct">{pct}%</span>
		</span>
	</footer>
</article>

<style>
	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		min-height: 180px;
		padding: 1.25rem 1.35rem;
		border-radius: 22px;
		background:
			linear-gradient(148deg, rgba(255, 255, 255, 0.12), transparent 42%),
			var(--glass-bg);
		border: 1px solid var(--glass-border);
		box-shadow: var(--glass-shadow);
		backdrop-filter: blur(16px) saturate(135%);
		color: var(--text);
		/* emerge from depth */
		animation: emerge var(--dur-slow) var(--ease-out) both;
		animation-delay: var(--delay);
		will-change: transform, opacity;
		overflow: hidden;
		transition:
			transform var(--dur) var(--ease-out),
			border-color var(--dur-fast) var(--ease-soft),
			box-shadow var(--dur) var(--ease-out);
	}
	.card:hover {
		transform: translateY(-4px);
		border-color: rgba(183, 246, 238, 0.3);
		box-shadow:
			0 32px 90px rgba(3, 10, 17, 0.4),
			0 0 50px rgba(99, 224, 207, 0.07),
			inset 0 1px 0 rgba(255, 255, 255, 0.17);
	}
	.card::before {
		/* faint per-card tint accent on the left edge */
		content: '';
		position: absolute;
		left: 1.25rem;
		right: 1.25rem;
		top: 0;
		height: 2px;
		border-radius: 2px;
		background: color-mix(in srgb, var(--accent) 60%, var(--tint));
		opacity: 0.8;
	}
	.media {
		position: relative;
		height: 150px;
		margin: -1.25rem -1.35rem 0.1rem;
		overflow: hidden;
		background: rgba(4, 12, 18, 0.22);
	}
	.media img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		opacity: 0;
		animation: reveal-image var(--dur-slow) var(--ease-out) forwards;
	}
	.media-shade {
		position: absolute;
		inset: 0;
		background: linear-gradient(180deg, transparent 42%, rgba(27, 39, 48, 0.92));
		pointer-events: none;
	}
	header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 0.6rem;
	}
	h3 {
		font-family: var(--font-display);
		font-weight: 650;
		font-size: 1.28rem;
		color: var(--text-strong);
		letter-spacing: 0.035em;
		text-transform: uppercase;
	}
	.cat {
		font-size: 0.68rem;
		color: var(--accent);
		white-space: nowrap;
		text-transform: uppercase;
		letter-spacing: 0.09em;
	}
	.metrics {
		display: flex;
		gap: 1.6rem;
	}
	.metric {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}
	.label {
		font-size: 0.7rem;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.value {
		font-family: var(--font-display);
		font-size: 1.16rem;
		font-weight: 580;
		color: var(--text-strong);
		font-variant-numeric: tabular-nums;
	}
	.value.low {
		color: var(--danger);
	}
	footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.8rem;
		margin-top: auto;
	}
	.reason {
		font-size: 0.78rem;
		color: var(--text-soft);
	}
	.conf {
		display: inline-flex;
		align-items: center;
		gap: 0.45rem;
	}
	.bar {
		display: block;
		width: 46px;
		height: 3px;
		border-radius: 3px;
		background: rgba(160, 180, 195, 0.12);
		position: relative;
		overflow: hidden;
	}
	.bar::after {
		content: '';
		position: absolute;
		inset: 0;
		width: var(--w);
		background: linear-gradient(90deg, var(--accent-soft), var(--accent));
	}
	.pct {
		font-size: 0.72rem;
		color: var(--text-soft);
		font-variant-numeric: tabular-nums;
	}

	/* Style hints — typography/emphasis, not boxes. */
	.hint-primary h3 {
		font-size: 1.34rem;
	}
	.hint-primary {
		background:
			linear-gradient(145deg, rgba(131, 234, 221, 0.15), transparent 48%),
			var(--glass-bg);
	}
	.hint-visual {
		min-height: 168px;
	}
	.hint-quiet {
		background: rgba(59, 75, 85, 0.38);
	}
	.hint-quiet h3 {
		font-weight: 400;
		color: var(--text);
	}
	.hint-dense {
		padding: 0.85rem 0.95rem;
		gap: 0.6rem;
	}
	.hint-dense h3 {
		font-size: 1.02rem;
	}

	@keyframes emerge {
		from {
			opacity: 0;
			transform: translateY(18px) scale(0.97);
			filter: blur(6px);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
			filter: blur(0);
		}
	}
	@keyframes reveal-image {
		from {
			opacity: 0;
			transform: scale(1.04);
			filter: blur(8px);
		}
		to {
			opacity: 0.88;
			transform: scale(1);
			filter: blur(0);
		}
	}
</style>
