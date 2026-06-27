<script lang="ts">
	// Layer 3 — product card. Появляется из глубины (не из модалки), со
	// стаггером по индексу. Стиль варьируется по card_style_hint.
	// Image upload zone supports paste (Ctrl+V), drag/drop, and click-to-browse.
	import { t } from '$lib/i18n';
	import { saveItemImage } from '$lib/tauri/commands';
	import type { ProductSearchResult } from '$lib/types';

	let {
		result,
		index = 0,
		imageUrl = null,
		onUploaded
	}: {
		result: ProductSearchResult;
		index?: number;
		imageUrl?: string | null;
		onUploaded?: (itemId: string) => void;
	} = $props();

	const item = $derived(result.item);
	const pct = $derived(Math.round(result.confidence * 100));
	const delay = $derived(Math.min(index * 55, 600));

	// Local upload state. We don't render spinners / errors globally because
	// multiple cards can be uploading at once — each card owns its own state.
	let uploading = $state(false);
	let uploadError = $state<string | null>(null);
	let dragOver = $state(false);

	// 5 MB matches the Rust limit in commands::inventory::save_item_image.
	const MAX_BYTES = 5 * 1024 * 1024;

	function fileToBase64(file: File): Promise<string> {
		return new Promise((resolve, reject) => {
			const reader = new FileReader();
			reader.onload = () => {
				const result = reader.result;
				if (typeof result !== 'string') return reject(new Error('reader returned non-string'));
				// strip the "data:...;base64," prefix — Rust expects raw base64.
				const comma = result.indexOf(',');
				resolve(comma >= 0 ? result.slice(comma + 1) : result);
			};
			reader.onerror = () => reject(reader.error ?? new Error('FileReader failed'));
			reader.readAsDataURL(file);
		});
	}

	async function uploadFromFile(file: File) {
		if (!file.type.startsWith('image/')) {
			uploadError = t('card.upload.notImage');
			return;
		}
		if (file.size > MAX_BYTES) {
			uploadError = t('card.upload.tooBig');
			return;
		}
		uploadError = null;
		uploading = true;
		try {
			const base64 = await fileToBase64(file);
			await saveItemImage(result.item_id, base64);
			onUploaded?.(result.item_id);
		} catch (e) {
			uploadError = String(e);
		} finally {
			uploading = false;
		}
	}

	async function onPaste(ev: ClipboardEvent) {
		const items = ev.clipboardData?.items;
		if (!items) return;
		for (const it of items) {
			if (it.kind === 'file' && it.type.startsWith('image/')) {
				const f = it.getAsFile();
				if (f) {
					ev.preventDefault();
					await uploadFromFile(f);
					return;
				}
			}
		}
	}

	function onDragOver(ev: DragEvent) {
		ev.preventDefault();
		dragOver = true;
	}
	function onDragLeave() {
		dragOver = false;
	}
	async function onDrop(ev: DragEvent) {
		ev.preventDefault();
		dragOver = false;
		const file = ev.dataTransfer?.files?.[0];
		if (file) await uploadFromFile(file);
	}

	let fileInput: HTMLInputElement;
	function onPickClick() {
		fileInput?.click();
	}
	async function onFileChange(ev: Event) {
		const target = ev.target as HTMLInputElement;
		const file = target.files?.[0];
		if (file) await uploadFromFile(file);
		// reset so picking the same file twice still fires `change`
		target.value = '';
	}
</script>

<article
	class="card hint-{result.card_style_hint}"
	class:has-image={imageUrl}
	style="--delay: {delay}ms; --tint: {item.card_color || 'transparent'}"
>
	<!-- Image zone: shows current image, or a drop/paste hint when missing.
	     Focused (tabindex=0) so Ctrl+V anywhere while this card has focus
	     attaches a clipboard image to this item. -->
	<div
		class="image-zone"
		class:has-image={!!imageUrl}
		class:drag-over={dragOver}
		class:uploading
		role="button"
		tabindex="0"
		aria-label={t('card.upload.aria', { title: result.title })}
		onpaste={onPaste}
		ondragover={onDragOver}
		ondragleave={onDragLeave}
		ondrop={onDrop}
		onclick={onPickClick}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				e.preventDefault();
				onPickClick();
			}
		}}
	>
		{#if imageUrl}
			<img src={imageUrl} alt={result.title} loading="lazy" />
			<div class="overlay">
				<span class="overlay-label">{t('card.upload.replace')}</span>
			</div>
		{:else}
			<div class="placeholder">
				<span class="ph-icon" aria-hidden="true">⤴</span>
				<span class="ph-text">{t('card.upload.hint')}</span>
				<span class="ph-sub">{t('card.upload.hintSub')}</span>
			</div>
		{/if}

		{#if uploading}
			<div class="overlay uploading-overlay">
				<span class="spinner" aria-hidden="true"></span>
				<span>{t('card.upload.uploading')}</span>
			</div>
		{/if}
	</div>

	<input
		bind:this={fileInput}
		type="file"
		accept="image/*"
		onchange={onFileChange}
		hidden
	/>

	{#if uploadError}
		<p class="err">{uploadError}</p>
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
		gap: 0.9rem;
		padding: 1.1rem 1.2rem;
		border-radius: 16px;
		background: var(--glass-bg);
		border: 1px solid var(--glass-border);
		box-shadow: var(--glass-shadow);
		backdrop-filter: blur(8px);
		color: var(--text);
		animation: emerge var(--dur-slow) var(--ease-out) both;
		animation-delay: var(--delay);
		will-change: transform, opacity;
	}
	.card.has-image {
		padding-top: 0.7rem;
	}
	.image-zone {
		margin: -0.4rem -0.5rem 0;
		border-radius: 12px;
		overflow: hidden;
		aspect-ratio: 16 / 10;
		background: rgba(0, 0, 0, 0.25);
		box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.04);
		position: relative;
		cursor: pointer;
		outline: none;
		transition: box-shadow var(--dur) var(--ease-out);
	}
	.image-zone:focus-visible {
		box-shadow: inset 0 0 0 2px var(--accent);
	}
	.image-zone.drag-over {
		box-shadow: inset 0 0 0 2px var(--accent), 0 0 30px -5px var(--accent-glow);
		background: rgba(0, 0, 0, 0.4);
	}
	.image-zone.uploading {
		cursor: progress;
	}
	.image-zone img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
		transition: transform var(--dur-slow) var(--ease-out);
	}
	.image-zone.has-image:hover img {
		transform: scale(1.03);
	}
	.overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: linear-gradient(180deg, transparent, rgba(0, 0, 0, 0.55));
		opacity: 0;
		transition: opacity var(--dur) var(--ease-out);
		color: var(--text-strong);
		font-size: 0.82rem;
		letter-spacing: 0.04em;
		pointer-events: none;
	}
	.image-zone.has-image:hover .overlay,
	.image-zone:focus-visible .overlay {
		opacity: 1;
	}
	.uploading-overlay {
		opacity: 1;
		background: rgba(0, 0, 0, 0.6);
		font-size: 0.85rem;
		gap: 0.5rem;
	}
	.spinner {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 2px solid rgba(255, 255, 255, 0.2);
		border-top-color: var(--accent);
		animation: spin 0.7s linear infinite;
	}
	.placeholder {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.25rem;
		color: var(--text-faint);
		text-align: center;
		padding: 0.6rem;
		border: 1.5px dashed rgba(255, 255, 255, 0.12);
		border-radius: 12px;
		transition: border-color var(--dur) var(--ease-out), color var(--dur) var(--ease-out);
	}
	.image-zone:hover .placeholder,
	.image-zone:focus-visible .placeholder,
	.image-zone.drag-over .placeholder {
		border-color: var(--accent);
		color: var(--text-soft);
	}
	.ph-icon {
		font-size: 1.5rem;
		line-height: 1;
	}
	.ph-text {
		font-size: 0.82rem;
		color: var(--text);
	}
	.ph-sub {
		font-size: 0.7rem;
		opacity: 0.7;
	}
	.err {
		font-size: 0.75rem;
		color: var(--danger);
		margin: -0.3rem 0 0;
	}

	.card::before {
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
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>