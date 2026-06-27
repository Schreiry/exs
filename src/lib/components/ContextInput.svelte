<script lang="ts">
	// Layer 2 — input surface. Главный экран сам является полем ввода.
	// Текст имеет лёгкое оптическое свечение и глубину, но читаемость — свята.
	import { t } from '$lib/i18n';

	let {
		value = $bindable(''),
		compact = false,
		onsubmit
	}: { value?: string; compact?: boolean; onsubmit: (q: string) => void } = $props();

	let el: HTMLTextAreaElement | null = $state(null);

	function handleKey(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			onsubmit(value);
		}
	}

	// Auto-grow within reason.
	function grow() {
		if (!el) return;
		el.style.height = 'auto';
		el.style.height = Math.min(el.scrollHeight, 160) + 'px';
	}

	$effect(() => {
		if (el && !compact) el.focus();
	});
</script>

<div class="field" class:compact>
	<textarea
		bind:this={el}
		bind:value
		rows="1"
		spellcheck="false"
		placeholder={t('input.placeholder')}
		onkeydown={handleKey}
		oninput={grow}
	></textarea>
	<div class="hint">{t('input.hint')}</div>
</div>

<style>
	.field {
		width: min(720px, 92vw);
		transition: transform var(--dur) var(--ease-out);
	}
	textarea {
		width: 100%;
		resize: none;
		overflow: hidden;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-strong);
		font-family: var(--font-display);
		font-weight: 300;
		font-size: clamp(1.5rem, 3.4vw, 2.4rem);
		line-height: 1.35;
		letter-spacing: 0.01em;
		text-align: center;
		caret-color: var(--accent);
		text-shadow: 0 1px 18px var(--accent-glow);
		transition: font-size var(--dur) var(--ease-out);
	}
	textarea::placeholder {
		color: var(--text-faint);
		text-shadow: none;
	}
	.compact textarea {
		font-size: clamp(1.05rem, 2vw, 1.3rem);
		text-align: left;
	}
	.hint {
		margin-top: 0.7rem;
		text-align: center;
		font-size: 0.8rem;
		color: var(--text-faint);
		opacity: 0.7;
		transition: opacity var(--dur) var(--ease-soft);
	}
	.compact .hint {
		display: none;
	}
	/* Subtle animated underline that breathes — a quiet "alive" cue. */
	.field::after {
		content: '';
		display: block;
		height: 1px;
		margin: 1rem auto 0;
		width: 38%;
		background: linear-gradient(90deg, transparent, var(--accent-soft), transparent);
		opacity: 0.5;
		animation: breathe 5s ease-in-out infinite;
	}
	.compact.field::after {
		display: none;
	}
	@keyframes breathe {
		0%,
		100% {
			opacity: 0.25;
			width: 32%;
		}
		50% {
			opacity: 0.6;
			width: 44%;
		}
	}
</style>
