<script lang="ts">
	// The screen itself is the input. There is no text box. The visible layer
	// is the user's draft rendered as large display typography that floats in
	// the void; the textarea is invisible but functional (caret, selection,
	// paste) so keyboard capture feels native.
	//
	// Two visual modes:
	//   - idle:    no draft → poetic placeholder hint, ghost-blink caret
	//   - writing: draft present → large display text, glass ripple on submit
	import { t } from '$lib/i18n';
	import type { ContextFileMetadata } from '$lib/types';

	let {
		value = $bindable(''),
		busy = false,
		attachments = [],
		onremoveattachment,
		onsubmit
	}: {
		value?: string;
		busy?: boolean;
		attachments?: ContextFileMetadata[];
		onremoveattachment?: (selectionId: string) => void;
		onsubmit: (q: string) => void;
	} = $props();

	let el: HTMLTextAreaElement | null = $state(null);
	let focused = $state(false);
	let submitting = $state(false);

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
			if (value.trim() && !busy) {
				submitting = true;
				onsubmit(value);
				// brief visual settle before re-render
				setTimeout(() => (submitting = false), 480);
			}
		}
	}

	// Auto-grow: textarea size to its content (invisible element, but height
	// matters for proper caret placement and scroll-into-view).
	function grow() {
		if (!el) return;
		el.style.height = 'auto';
		el.style.height = Math.min(el.scrollHeight, 320) + 'px';
	}

	$effect(() => {
		if (!el) return;
		requestAnimationFrame(() => focusInput());
	});
</script>

<!-- The visible "draft" — large display text that lives in the void, not in a box. -->
<div
	class="voice"
	class:writing={value.length > 0}
	class:focused
	class:busy
	class:submitting
	aria-hidden="true"
>
	{#if value.length > 0}
		<div class="draft">
			{#each value.split(/(\s+)/) as token, i (i)}
				<span class="word" style="--i: {i}">{token === ' ' ? ' ' : token}</span>
			{/each}
			<span class="caret" aria-hidden="true"></span>
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
	{:else}
		<!-- Idle placeholder: the cursor itself is the only UI. The text fades in
		     on first keystroke. Subtle signal dot animates when AI is busy. -->
		<div class="placeholder">
			<span class="placeholder-kicker">
				{#if busy}
					<span class="signal" aria-hidden="true"></span>
					{t('state.thinking')}
				{:else}
					{t('input.placeholder')}
				{/if}
			</span>
			<span class="placeholder-caret" aria-hidden="true"></span>
		</div>
	{/if}
</div>

<!-- The hidden, functional input. No chrome. The visible "draft" above mirrors it. -->
<textarea
	bind:this={el}
	bind:value
	class="ghost"
	rows="1"
	spellcheck="false"
	aria-label={t('input.placeholder')}
	placeholder={t('input.placeholder')}
	onkeydown={handleKey}
	oninput={grow}
	onfocus={() => (focused = true)}
	onblur={() => (focused = false)}
></textarea>

<style>
	/* The "voice" — the visible draft area. Centered in the void, no chrome. */
	.voice {
		position: relative;
		width: min(1080px, 88vw);
		min-height: clamp(8rem, 18vh, 14rem);
		display: grid;
		place-items: center;
		text-align: center;
		pointer-events: none;
		transition:
			transform var(--dur-slow) var(--ease-out),
			opacity var(--dur) var(--ease-soft);
	}
	.voice.focused .draft,
	.voice.focused .placeholder {
		text-shadow:
			0 1px 0 rgba(255, 255, 255, 0.04),
			0 2px 24px var(--accent-glow),
			0 12px 60px rgba(0, 0, 0, 0.3);
	}

	/* — IDLE: the void itself is the input surface. Only the caret lives here. */
	.placeholder {
		display: grid;
		justify-items: center;
		gap: 0.6rem;
		color: var(--text-faint);
		transition: color var(--dur) var(--ease-soft);
	}
	.placeholder-kicker {
		font-family: var(--font-display);
		font-size: clamp(1.05rem, 2vw, 1.4rem);
		font-weight: 400;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		display: inline-flex;
		align-items: center;
		gap: 0.7rem;
	}
	.voice.busy .placeholder-kicker {
		color: var(--accent);
	}
	.signal {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 18px var(--accent);
		animation: pulse 1.15s ease-in-out infinite;
	}
	.placeholder-caret {
		display: inline-block;
		width: 1.5px;
		height: clamp(1.6rem, 3vh, 2.4rem);
		background: var(--text-faint);
		opacity: 0.55;
		animation: blink 1.05s steps(2) infinite;
	}

	/* — WRITING: the user's draft, rendered as display text in the void. */
	.draft {
		font-family: var(--font-display);
		font-size: clamp(2.2rem, 5.6vw, 4.6rem);
		font-weight: 540;
		line-height: 1.04;
		letter-spacing: 0.025em;
		color: var(--text-strong);
		text-transform: uppercase;
		text-wrap: balance;
		max-width: 100%;
		word-break: break-word;
		display: inline;
	}
	.word {
		display: inline-block;
		opacity: 0;
		transform: translateY(8px);
		filter: blur(4px);
		animation: word-rise var(--dur) var(--ease-out) forwards;
		animation-delay: calc(var(--i, 0) * 12ms);
	}
	.caret {
		display: inline-block;
		width: 2px;
		height: 0.92em;
		margin-left: 0.08em;
		vertical-align: text-bottom;
		background: var(--accent);
		box-shadow: 0 0 12px var(--accent-glow);
		animation: blink 1.05s steps(2) infinite;
	}

	/* Attachments — minimal, sit below the draft. */
	.attachments {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		margin-top: 1.4rem;
		max-width: 100%;
		pointer-events: auto;
	}
	.attachment {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.32rem 0.7rem;
		border: 1px solid var(--glass-border);
		border-radius: 999px;
		background: var(--glass-bg);
		backdrop-filter: blur(14px);
		color: var(--text-soft);
		font-family: var(--font-display);
		font-size: 0.72rem;
		font-weight: 540;
		letter-spacing: 0.04em;
		pointer-events: auto;
		transition: border-color var(--dur-fast), color var(--dur-fast);
	}
	.attachment:hover {
		border-color: var(--accent-glow);
		color: var(--text-strong);
	}
	.attachment-mark {
		color: var(--accent);
	}
	.attachment-name {
		max-width: 11rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Submitting: brief zoom-out + fade. The draft "lets go" of the void. */
	.voice.submitting .draft {
		animation: submit-release var(--dur) var(--ease-out) forwards;
	}

	/* — HIDDEN TEXTAREA. Receives all keyboard input. — */
	.ghost {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		opacity: 0;
		color: transparent;
		background: transparent;
		border: none;
		outline: none;
		resize: none;
		caret-color: transparent;
		font: inherit;
		pointer-events: auto;
		z-index: 1;
	}
	.ghost::placeholder {
		color: transparent;
	}

	@keyframes word-rise {
		from {
			opacity: 0;
			transform: translateY(10px) scale(0.97);
			filter: blur(6px);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
			filter: blur(0);
		}
	}
	@keyframes submit-release {
		from {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
		to {
			opacity: 0;
			transform: translateY(-30px) scale(0.96);
		}
	}
	@keyframes blink {
		0%,
		50% {
			opacity: 1;
		}
		51%,
		100% {
			opacity: 0;
		}
	}
	@keyframes pulse {
		0%,
		100% {
			opacity: 0.4;
			transform: scale(0.85);
		}
		50% {
			opacity: 1;
			transform: scale(1.25);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.word,
		.caret,
		.placeholder-caret,
		.signal,
		.voice.submitting .draft {
			animation: none;
		}
		.word {
			opacity: 1;
			transform: none;
			filter: none;
		}
	}
</style>