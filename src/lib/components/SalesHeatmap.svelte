<script lang="ts">
	// Sales Heatmap — "როდის ყიდიან"
	// 7 rows (weekdays) × 24 columns (hours) grid of cell intensity.
	// Pure CSS: color via opacity over a graphite base; no chart library.
	// Adds a peak insight line ("პიკი: ...").
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { isTauri, getSalesHeatmap } from '$lib/tauri/commands';
	import type { HeatmapCell } from '$lib/types';

	type PeriodDays = 7 | 30 | 90 | 0; // 0 = all time

	let cells = $state<HeatmapCell[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let period = $state<PeriodDays>(30);

	// Lookup map for O(1) cell access.
	const cellMap = $derived.by(() => {
		const m = new Map<string, HeatmapCell>();
		for (const c of cells) m.set(`${c.weekday}:${c.hour}`, c);
		return m;
	});

	// Max revenue in current view — drives intensity scale.
	const maxRevenue = $derived(cells.reduce((m, c) => Math.max(m, c.revenue), 0));
	const totalRevenue = $derived(cells.reduce((s, c) => s + c.revenue, 0));

	// Peak insight — the (weekday, hour) with highest revenue.
	const peak = $derived.by(() => {
		if (cells.length === 0) return null;
		let best = cells[0];
		for (const c of cells) if (c.revenue > best.revenue) best = c;
		if (best.revenue <= 0 || totalRevenue <= 0) return null;
		return {
			weekday: best.weekday,
			hour: best.hour,
			revenue: best.revenue,
			units: best.units,
			pct: Math.round((best.revenue / totalRevenue) * 100)
		};
	});

	// Hover tooltip state — single source of truth, no per-cell components.
	let hoverCell = $state<{
		weekday: number;
		hour: number;
		x: number;
		y: number;
		revenue: number;
		units: number;
		intensity: number;
	} | null>(null);

	function sinceIsoFor(days: PeriodDays): string | undefined {
		if (days === 0) return undefined;
		const d = new Date();
		d.setUTCDate(d.getUTCDate() - days);
		return d.toISOString().slice(0, 10);
	}

	async function load(p: PeriodDays) {
		if (!isTauri()) {
			loading = false;
			return;
		}
		loading = true;
		error = null;
		try {
			cells = await getSalesHeatmap(sinceIsoFor(p));
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		load(period);
	});

	function setPeriod(p: PeriodDays) {
		if (p === period) return;
		period = p;
		load(p);
	}

	function intensity(revenue: number): number {
		if (maxRevenue <= 0) return 0;
		return Math.min(1, Math.max(0.04, revenue / maxRevenue));
	}

	function onCellEnter(e: MouseEvent | FocusEvent, weekday: number, hour: number) {
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const containerRect = (target.closest('.heatmap-wrap') as HTMLElement).getBoundingClientRect();
		const c = cellMap.get(`${weekday}:${hour}`);
		hoverCell = {
			weekday,
			hour,
			x: rect.left - containerRect.left + rect.width / 2,
			y: rect.top - containerRect.top,
			revenue: c?.revenue ?? 0,
			units: c?.units ?? 0,
			intensity: intensity(c?.revenue ?? 0)
		};
	}

	function onCellLeave() {
		hoverCell = null;
	}

	function fmtMoney(n: number): string {
		return n.toFixed(2);
	}

	// Weekday order: starting Monday (more natural for business).
	// SQLite strftime('%w') returns 0=Sun..6=Sat; we want Mon..Sun.
	const displayWeekdays = [1, 2, 3, 4, 5, 6, 0];
	const weekdayKeys = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;

	function weekdayLabel(weekday: number): string {
		// t('analytics.heatmap.weekdays') returns array; index matches strftime order (0=Sun).
		const arr = t('analytics.heatmap.weekdays').split('|');
		return arr[weekday] ?? '';
	}
</script>

<section class="block heatmap-block">
	<header class="block-head">
		<div>
			<h2>{t('analytics.heatmap.title')}</h2>
			<p class="hint">{t('analytics.heatmap.hint')}</p>
		</div>
		<div class="period-switch" role="tablist">
			<button
				class:active={period === 7}
				onclick={() => setPeriod(7)}
				aria-current={period === 7 ? 'true' : undefined}
				>{t('analytics.heatmap.periods.7')}</button
			>
			<button
				class:active={period === 30}
				onclick={() => setPeriod(30)}
				aria-current={period === 30 ? 'true' : undefined}
				>{t('analytics.heatmap.periods.30')}</button
			>
			<button
				class:active={period === 90}
				onclick={() => setPeriod(90)}
				aria-current={period === 90 ? 'true' : undefined}
				>{t('analytics.heatmap.periods.90')}</button
			>
			<button
				class:active={period === 0}
				onclick={() => setPeriod(0)}
				aria-current={period === 0 ? 'true' : undefined}
				>{t('analytics.heatmap.periods.all')}</button
			>
		</div>
	</header>

	{#if loading}
		<div class="status pulse">{t('analytics.loading')}</div>
	{:else if error}
		<div class="status err">{t('state.error')} — {error}</div>
	{:else if cells.length === 0}
		<div class="empty">{t('analytics.heatmap.empty')}</div>
	{:else}
		<div class="heatmap-wrap">
			<!-- Hour axis (top) -->
			<div class="axis top" aria-hidden="true">
				<div class="axis-spacer"></div>
				{#each Array(24) as _, h (h)}
					{#if h % 2 === 0}
						<div class="axis-tick">{String(h).padStart(2, '0')}</div>
					{:else}
						<div class="axis-tick muted"></div>
					{/if}
				{/each}
			</div>

			<!-- Grid body -->
			<div
				class="heatmap"
				role="img"
				aria-label={t('analytics.heatmap.title')}
			>
				{#each displayWeekdays as wd, wi (wd)}
					<div class="row">
						<div class="row-label">{weekdayLabel(wd)}</div>
						{#each Array(24) as _, h (h)}
							{@const c = cellMap.get(`${wd}:${h}`)}
							{@const v = c?.revenue ?? 0}
							{@const k = intensity(v)}
							<button
								type="button"
								class="cell"
								style="--k: {k}"
								onmouseenter={(e) => onCellEnter(e, wd, h)}
								onmouseleave={onCellLeave}
								onfocus={(e) => onCellEnter(e, wd, h)}
								onblur={onCellLeave}
								aria-label={t('analytics.heatmap.cellHint', {
									weekday: weekdayLabel(wd),
									hour: String(h).padStart(2, '0'),
									units: c?.units ?? 0,
									revenue: fmtMoney(v)
								})}
							></button>
						{/each}
					</div>
				{/each}
			</div>

			<!-- Tooltip -->
			{#if hoverCell}
				<div
					class="tip"
					style="left: {hoverCell.x}px; top: {hoverCell.y}px; --k: {hoverCell.intensity}"
				>
					<div class="tip-time">{weekdayLabel(hoverCell.weekday)} {String(hoverCell.hour).padStart(2, '0')}:00</div>
					<div class="tip-rev">{fmtMoney(hoverCell.revenue)} {t('units.currency')}</div>
					<div class="tip-units">×{hoverCell.units}</div>
				</div>
			{/if}
		</div>

		<!-- Peak insight line -->
		{#if peak}
			<p class="peak">
				<span class="peak-label">{t('analytics.heatmap.peakLabel')}:</span>
				{t('analytics.heatmap.peakHint', {
					weekday: weekdayLabel(peak.weekday),
					hour: String(peak.hour).padStart(2, '0'),
					pct: peak.pct
				})}
			</p>
		{/if}

		<!-- Color scale legend -->
		<div class="scale" aria-hidden="true">
			<span class="scale-label">{t('analytics.heatmap.scale')}</span>
			<div class="scale-bar"></div>
		</div>
	{/if}
</section>

<style>
	.heatmap-block {
		display: flex;
		flex-direction: column;
		gap: 0.85rem;
		padding: 1.1rem 1.2rem;
		border-radius: 16px;
		background: var(--glass-bg);
		border: 1px solid var(--glass-border);
		box-shadow: var(--glass-shadow);
		backdrop-filter: blur(8px);
		animation: rise var(--dur-slow) var(--ease-out) both;
	}
	.block-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 1rem;
		flex-wrap: wrap;
	}
	.block-head h2 {
		font-family: var(--font-display);
		font-weight: 400;
		font-size: 1.05rem;
		color: var(--text-strong);
		margin: 0;
	}
	.hint {
		font-size: 0.78rem;
		color: var(--text-faint);
		margin: 0.25rem 0 0;
	}

	.period-switch {
		display: inline-flex;
		gap: 0.25rem;
		padding: 0.18rem;
		border-radius: 999px;
		background: rgba(20, 26, 32, 0.45);
		border: 1px solid var(--glass-border);
	}
	.period-switch button {
		appearance: none;
		background: transparent;
		border: 0;
		color: var(--text-soft);
		font-size: 0.74rem;
		letter-spacing: 0.04em;
		padding: 0.32rem 0.7rem;
		border-radius: 999px;
		cursor: pointer;
		transition: background var(--dur) var(--ease-out), color var(--dur) var(--ease-out);
	}
	.period-switch button:hover {
		color: var(--text);
	}
	.period-switch button.active {
		background: var(--accent-soft);
		color: var(--text-strong);
	}

	.status,
	.empty {
		font-size: 0.9rem;
		color: var(--text-soft);
		padding: 1.5rem 0;
		text-align: center;
	}
	.status.err {
		color: var(--danger);
	}
	.pulse {
		animation: pulse 1.4s ease-in-out infinite;
	}

	.heatmap-wrap {
		position: relative;
		overflow-x: auto;
		padding-bottom: 0.25rem;
	}

	/* Hour axis (top) */
	.axis {
		display: grid;
		grid-template-columns: 2.5rem repeat(24, minmax(0, 1fr));
		gap: 3px;
		margin-bottom: 0.4rem;
		min-width: 38rem;
	}
	.axis-spacer {
		/* aligns with row labels width */
		/* (intentionally empty — width set by grid template) */
		grid-column: 1;
	}
	.axis-tick {
		font-size: 0.62rem;
		color: var(--text-faint);
		font-variant-numeric: tabular-nums;
		text-align: left;
		padding-left: 0.1rem;
		line-height: 1;
	}
	.axis-tick.muted {
		visibility: hidden;
	}

	/* Grid */
	.heatmap {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 38rem;
	}
	.row {
		display: grid;
		grid-template-columns: 2.5rem repeat(24, minmax(0, 1fr));
		gap: 3px;
	}
	.row-label {
		font-size: 0.72rem;
		color: var(--text-soft);
		display: flex;
		align-items: center;
		font-variant-numeric: tabular-nums;
	}

	/* The cell — pure CSS intensity via opacity on the accent color */
	.cell {
		appearance: none;
		border: 0;
		padding: 0;
		height: 22px;
		border-radius: 3px;
		cursor: pointer;
		background: color-mix(
			in srgb,
			var(--accent) calc(var(--k) * 100%),
			rgba(160, 180, 195, 0.06)
		);
		transition:
			transform var(--dur) var(--ease-out),
			box-shadow var(--dur) var(--ease-out),
			filter var(--dur) var(--ease-out);
		outline: none;
	}
	.cell:hover,
	.cell:focus-visible {
		transform: scale(1.18);
		box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 60%, transparent);
		filter: brightness(1.15);
		z-index: 1;
	}

	/* Tooltip */
	.tip {
		position: absolute;
		transform: translate(-50%, calc(-100% - 8px));
		padding: 0.55rem 0.75rem;
		background: rgba(15, 22, 28, 0.92);
		border: 1px solid var(--glass-border);
		border-radius: 8px;
		backdrop-filter: blur(8px);
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
		pointer-events: none;
		min-width: 9rem;
		font-size: 0.78rem;
		color: var(--text);
		white-space: nowrap;
		animation: tip-in 0.14s var(--ease-out) both;
		z-index: 10;
	}
	.tip::after {
		content: '';
		position: absolute;
		left: 50%;
		bottom: -5px;
		transform: translateX(-50%) rotate(45deg);
		width: 8px;
		height: 8px;
		background: rgba(15, 22, 28, 0.92);
		border-right: 1px solid var(--glass-border);
		border-bottom: 1px solid var(--glass-border);
	}
	.tip-time {
		font-size: 0.7rem;
		color: var(--text-faint);
		margin-bottom: 0.15rem;
		letter-spacing: 0.04em;
	}
	.tip-rev {
		font-family: var(--font-display);
		font-size: 0.95rem;
		color: var(--text-strong);
		font-variant-numeric: tabular-nums;
	}
	.tip-units {
		font-size: 0.7rem;
		color: var(--text-soft);
		margin-top: 0.15rem;
		font-variant-numeric: tabular-nums;
	}

	/* Peak insight */
	.peak {
		font-size: 0.84rem;
		color: var(--text-soft);
		margin: 0;
		padding-top: 0.4rem;
		border-top: 1px solid rgba(160, 180, 195, 0.08);
		line-height: 1.5;
	}
	.peak-label {
		color: var(--accent);
		font-weight: 500;
		margin-right: 0.4rem;
		text-transform: uppercase;
		font-size: 0.72rem;
		letter-spacing: 0.06em;
	}

	/* Color scale legend */
	.scale {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-size: 0.7rem;
		color: var(--text-faint);
	}
	.scale-label {
		letter-spacing: 0.04em;
	}
	.scale-bar {
		flex: 1;
		max-width: 14rem;
		height: 6px;
		border-radius: 999px;
		background: linear-gradient(
			90deg,
			rgba(160, 180, 195, 0.06),
			color-mix(in srgb, var(--accent) 100%, transparent)
		);
	}

	@keyframes rise {
		from {
			opacity: 0;
			transform: translateY(10px);
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
	@keyframes tip-in {
		from {
			opacity: 0;
			transform: translate(-50%, calc(-100% - 2px));
		}
		to {
			opacity: 1;
			transform: translate(-50%, calc(-100% - 8px));
		}
	}

	@media (max-width: 640px) {
		.axis,
		.heatmap,
		.row {
			min-width: 30rem;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.cell,
		.tip,
		.heatmap-block {
			animation: none !important;
			transition: none !important;
		}
		.cell:hover,
		.cell:focus-visible {
			transform: none;
		}
	}
</style>