<script lang="ts">
	// Layer 3 — product card. Появляется из глубины (не из модалки), со
	// стаггером по индексу. Стиль варьируется по card_style_hint.
	import { t } from '$lib/i18n';
	import type { ProductSearchResult } from '$lib/types';

	let { result, index = 0 }: { result: ProductSearchResult; index?: number } = $props();

	const item = $derived(result.item);
	const pct = $derived(Math.round(result.confidence * 100));
	const delay = $derived(Math.min(index * 55, 600));
</script>

<article
	class="card hint-{result.card_style_hint}"
	style="--delay: {delay}ms; --tint: {item.card_color || 'transparent'}"
>
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
		gap: 0.9rem;
		padding: 1.1rem 1.2rem;
		border-radius: 16px;
		background: var(--glass-bg);
		border: 1px solid var(--glass-border);
		box-shadow: var(--glass-shadow);
		backdrop-filter: blur(8px);
		color: var(--text);
		/* emerge from depth */
		animation: emerge var(--dur-slow) var(--ease-out) both;
		animation-delay: var(--delay);
		will-change: transform, opacity;
	}
	.card::before {
		/* faint per-card tint accent on the left edge */
		content: '';
		position: absolute;
		left: 0;
		top: 14%;
		bottom: 14%;
		width: 2px;
		border-radius: 2px;
		background: color-mix(in srgb, var(--accent) 60%, var(--tint));
		opacity: 0.5;
	}
	header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 0.6rem;
	}
	h3 {
		font-family: var(--font-display);
		font-weight: 500;
		font-size: 1.18rem;
		color: var(--text-strong);
		letter-spacing: 0.01em;
	}
	.cat {
		font-size: 0.74rem;
		color: var(--text-faint);
		white-space: nowrap;
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
		font-size: 1.05rem;
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
		background: linear-gradient(180deg, rgba(45, 56, 64, 0.62), var(--glass-bg));
	}
	.hint-visual {
		min-height: 168px;
	}
	.hint-quiet {
		background: rgba(29, 34, 40, 0.4);
		box-shadow: none;
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
</style>
