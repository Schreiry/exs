<script lang="ts">
	// Layer 2 — input surface. Главный экран сам является полем ввода.
	// Текст имеет лёгкое оптическое свечение и глубину, но читаемость — свята.
	import { t } from '$lib/i18n';
	import type { ContextFileMetadata } from '$lib/types';

	let {
		value = $bindable(''),
		compact = false,
		busy = false,
		attachments = [],
		onremoveattachment,
		onsubmit
	}: {
		value?: string;
		compact?: boolean;
		busy?: boolean;
		attachments?: ContextFileMetadata[];
		onremoveattachment?: (selectionId: string) => void;
		onsubmit: (q: string) => void;
	} = $props();

	let el: HTMLTextAreaElement | null = $state(null);
	let focused = $state(false);

	export function focusInput(selectEnd = true) {
		if (!el) return;
		el.focus({ preventScroll: true });
		if (selectEnd) {
			const end = el.value.length;
			el.setSelectionRange(end, end);
		}
	}

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
		if (!el || compact) return;
		requestAnimationFrame(() => focusInput());
	});
</script>

<div class="field" class:compact class:focused class:busy>
	<div class="eyebrow">
		<span class="signal" aria-hidden="true"></span>
		<span>{busy ? t('state.thinking') : t('assistant.label')}</span>
	</div>
	{#if attachments.length}
		<div class="attachments" aria-label={t('files.label')}>
			{#each attachments as file (file.selection_id)}
				<button
					type="button"
					class="attachment"
					title={t('files.remove', { name: file.file_name })}
					aria-label={t('files.remove', { name: file.file_name })}
					onclick={() => onremoveattachment?.(file.selection_id)}
				>
					<span class="attachment-mark" aria-hidden="true">◇</span>
					<span class="attachment-name">{file.file_name}</span>
					<span class="attachment-remove" aria-hidden="true">×</span>
				</button>
			{/each}
		</div>
	{/if}
	<textarea
		bind:this={el}
		bind:value
		rows="1"
		spellcheck="false"
		aria-label={t('input.placeholder')}
		placeholder={t('input.placeholder')}
		onkeydown={handleKey}
		oninput={grow}
		onfocus={() => (focused = true)}
		onblur={() => (focused = false)}
	></textarea>
	<div class="meta">
		<span class="hint">{t('input.hint')}</span>
		<span class="shortcut">ALT + C</span>
	</div>
</div>

<style>
	.field {
		position: relative;
		width: min(980px, 90vw);
		padding: clamp(1.2rem, 2.4vw, 2rem) clamp(1rem, 3vw, 2.5rem) 1rem;
		border: 1px solid rgba(255, 255, 255, 0.16);
		border-radius: 28px;
		background:
			linear-gradient(145deg, rgba(255, 255, 255, 0.11), rgba(255, 255, 255, 0.025) 44%),
			rgba(30, 38, 47, 0.5);
		box-shadow:
			0 30px 90px rgba(0, 0, 0, 0.36),
			inset 0 1px 0 rgba(255, 255, 255, 0.19),
			inset 0 -1px 0 rgba(0, 0, 0, 0.28);
		backdrop-filter: blur(22px) saturate(145%);
		transform: translateZ(0);
		transition:
			border-color var(--dur-fast) var(--ease-soft),
			box-shadow var(--dur) var(--ease-out),
			transform var(--dur) var(--ease-out);
	}
	.field::before {
		content: '';
		position: absolute;
		inset: 0;
		z-index: -1;
		border-radius: inherit;
		background: radial-gradient(70% 120% at 50% 0%, rgba(123, 239, 226, 0.13), transparent 58%);
		opacity: 0.35;
		transition: opacity var(--dur) var(--ease-soft);
		pointer-events: none;
	}
	.field.focused {
		border-color: rgba(151, 244, 232, 0.38);
		box-shadow:
			0 34px 110px rgba(0, 0, 0, 0.42),
			0 0 80px rgba(76, 224, 207, 0.09),
			inset 0 1px 0 rgba(255, 255, 255, 0.25);
		transform: translateY(-2px);
	}
	.field.focused::before,
	.field.busy::before {
		opacity: 1;
	}
	.eyebrow,
	.meta {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.55rem;
		color: var(--text-soft);
		font-family: var(--font-display);
		font-size: 0.68rem;
		font-weight: 620;
		letter-spacing: 0.19em;
		text-transform: uppercase;
	}
	.eyebrow {
		margin-bottom: 0.6rem;
	}
	.attachments {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.45rem;
		max-width: 100%;
		margin: 0 auto 0.7rem;
		overflow-x: auto;
		scrollbar-width: none;
	}
	.attachments::-webkit-scrollbar {
		display: none;
	}
	.attachment {
		display: inline-flex;
		align-items: center;
		gap: 0.38rem;
		flex: 0 0 auto;
		max-width: 13rem;
		padding: 0.36rem 0.52rem;
		border: 1px solid rgba(135, 235, 222, 0.24);
		border-radius: 999px;
		background: rgba(100, 225, 208, 0.075);
		color: var(--text-soft);
		font-family: var(--font-display);
		font-size: 0.66rem;
		font-weight: 590;
		letter-spacing: 0.035em;
	}
	.attachment:hover {
		border-color: rgba(158, 246, 235, 0.4);
		color: var(--text-strong);
	}
	.attachment-mark {
		color: var(--accent);
	}
	.attachment-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.attachment-remove {
		font-size: 0.9rem;
		opacity: 0.62;
	}
	.signal {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 14px var(--accent);
	}
	.busy .signal {
		animation: signal 1.15s ease-in-out infinite;
	}
	textarea {
		width: 100%;
		min-height: 2.2em;
		max-height: 30vh;
		resize: none;
		overflow: hidden;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-strong);
		font-family: var(--font-display);
		font-weight: 590;
		font-size: clamp(2rem, 5.4vw, 4.9rem);
		line-height: 1.06;
		letter-spacing: 0.035em;
		text-align: center;
		text-transform: uppercase;
		caret-color: var(--accent);
		text-wrap: balance;
		text-shadow: 0 2px 34px rgba(113, 235, 220, 0.12);
		transition:
			font-size var(--dur) var(--ease-out),
			color var(--dur-fast) var(--ease-soft);
	}
	textarea::placeholder {
		color: rgba(223, 233, 239, 0.44);
		text-shadow: none;
	}
	.compact textarea {
		font-size: clamp(1.2rem, 2.2vw, 1.8rem);
	}
	.meta {
		justify-content: space-between;
		margin-top: 0.9rem;
		color: var(--text-faint);
		letter-spacing: 0.1em;
	}
	.compact .hint {
		display: none;
	}
	.shortcut {
		padding: 0.28rem 0.5rem;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 7px;
		background: rgba(255, 255, 255, 0.035);
	}
	@keyframes signal {
		0%,
		100% {
			opacity: 0.35;
			transform: scale(0.8);
		}
		50% {
			opacity: 1;
			transform: scale(1.35);
		}
	}
	@media (max-width: 620px) {
		.field {
			width: min(94vw, 980px);
			border-radius: 22px;
		}
		textarea {
			font-size: clamp(1.7rem, 9vw, 3rem);
		}
		.shortcut {
			display: none;
		}
		.meta {
			justify-content: center;
		}
	}
</style>
