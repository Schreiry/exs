<script lang="ts">
	// Layer 3 — analytics scene. One calm surface that appears in the void
	// when the user asks for an overview (top sellers, dead stock, categories,
	// recent activity). All data fetched in parallel from typed Tauri commands.
	// No charts — CSS-only bars keep the void aesthetic and respect perf rules.
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { isTauri } from '$lib/tauri/commands';
	import SalesHeatmap from './SalesHeatmap.svelte';
	import {
		getInventorySummary,
		getTopSellers,
		getDeadStock,
		getLowStockItems,
		getCategoryBreakdown,
		getSalesTimeseries,
		getAiCoverage,
		getRecentActivity,
		getStockOutForecast
	} from '$lib/tauri/commands';
	import type {
		InventorySummary,
		TopSeller,
		DeadStock,
		LowStockItem,
		CategoryBreakdown,
		TimeseriesPoint,
		AiCoverage,
		ActivityEntry,
		StockOutForecast,
		SalesBucket
	} from '$lib/types';

	type Bucket = SalesBucket;

	let inventory = $state<InventorySummary | null>(null);
	let topSellers = $state<TopSeller[]>([]);
	let deadStock = $state<DeadStock[]>([]);
	let lowStock = $state<LowStockItem[]>([]);
	let categories = $state<CategoryBreakdown[]>([]);
	let timeseries = $state<TimeseriesPoint[]>([]);
	let coverage = $state<AiCoverage | null>(null);
	let activity = $state<ActivityEntry[]>([]);
	let forecast = $state<StockOutForecast[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let bucket = $state<Bucket>('day');

	// Category revenue share — used for soft horizontal bars (0..1).
	const maxCategoryRevenue = $derived(
		categories.reduce((m, c) => Math.max(m, c.revenue), 0)
	);
	const maxTsRevenue = $derived(timeseries.reduce((m, p) => Math.max(m, p.revenue), 0));
	const totalCategoryRevenue = $derived(
		categories.reduce((s, c) => s + c.revenue, 0)
	);

	async function load(currentBucket: Bucket) {
		if (!isTauri()) {
			loading = false;
			error = null;
			return;
		}
		loading = true;
		error = null;
		try {
			const [
				inv,
				sellers,
				dead,
				low,
				cats,
				ts,
				cov,
				act,
				fc
			] = await Promise.all([
				getInventorySummary(3),
				getTopSellers(5),
				getDeadStock(5),
				getLowStockItems(3, 10),
				getCategoryBreakdown(),
				getSalesTimeseries(currentBucket),
				getAiCoverage(),
				getRecentActivity(8),
				getStockOutForecast(8, 3)
			]);
			inventory = inv;
			topSellers = sellers;
			deadStock = dead;
			lowStock = low;
			categories = cats;
			timeseries = ts;
			coverage = cov;
			activity = act;
			forecast = fc;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		load(bucket);
	});

	function setBucket(b: Bucket) {
		if (b === bucket) return;
		bucket = b;
		load(b);
	}

	function fmtMoney(n: number): string {
		return n.toFixed(2);
	}

	function coveragePct(part: number, total: number): number {
		if (!total) return 0;
		return Math.min(1, part / total);
	}
</script>

<div class="scene">
	<header class="head">
		<span class="kicker">{t('analytics.title')}</span>
		<p class="sub">{t('analytics.subtitle')}</p>
	</header>

	{#if loading}
		<div class="status pulse">{t('analytics.loading')}</div>
	{:else if error}
		<div class="status err">{t('state.error')} — {error}</div>
	{:else if !inventory}
		<div class="empty">{t('analytics.empty')}</div>
	{:else}
		<!-- KPI tiles ─────────────────────────────────────── -->
		<section class="kpis">
			<div class="kpi">
				<span class="label">{t('analytics.kpi.items')}</span>
				<span class="value">{inventory.item_count}</span>
			</div>
			<div class="kpi">
				<span class="label">{t('analytics.kpi.stockValue')}</span>
				<span class="value">{fmtMoney(inventory.stock_value_at_price)} {t('units.currency')}</span>
			</div>
			<div class="kpi">
				<span class="label">{t('analytics.kpi.revenue')}</span>
				<span class="value">{fmtMoney(inventory.total_revenue)} {t('units.currency')}</span>
			</div>
			<div class="kpi">
				<span class="label">{t('analytics.kpi.soldUnits')}</span>
				<span class="value">{inventory.total_sold}</span>
			</div>
			<div class="kpi" class:warn={inventory.low_stock_count > 0}>
				<span class="label">{t('analytics.kpi.lowStock')}</span>
				<span class="value">{inventory.low_stock_count}</span>
			</div>
			{#if coverage}
				<div class="kpi">
					<span class="label">{t('analytics.kpi.aiCoverage')}</span>
					<span class="value">{coverage.with_tags}/{coverage.total_items}</span>
				</div>
			{/if}
		</section>

		<!-- Sales time-series ───────────────────────────── -->
		<section class="block">
			<header class="block-head">
				<h2>{t('analytics.revenueLabel')}</h2>
				<div class="bucket-switch" role="tablist" aria-label={t('analytics.bucketLabel')}>
					<button
						class:active={bucket === 'day'}
						onclick={() => setBucket('day')}
						role="tab"
						aria-selected={bucket === 'day'}>{t('analytics.buckets.day')}</button
					>
					<button
						class:active={bucket === 'week'}
						onclick={() => setBucket('week')}
						role="tab"
						aria-selected={bucket === 'week'}>{t('analytics.buckets.week')}</button
					>
					<button
						class:active={bucket === 'month'}
						onclick={() => setBucket('month')}
						role="tab"
						aria-selected={bucket === 'month'}>{t('analytics.buckets.month')}</button
					>
				</div>
			</header>

			{#if timeseries.length === 0}
				<p class="muted">{t('analytics.empty')}</p>
			{:else}
				<ol class="ts">
					{#each timeseries as p (p.bucket)}
						<li class="ts-row">
							<span class="ts-label">{p.bucket}</span>
							<span class="ts-bar"
								><span class="ts-fill" style="--w: {maxTsRevenue
									? (p.revenue / maxTsRevenue) * 100
									: 0}%"></span></span
							>
							<span class="ts-meta">
								<span class="ts-rev">{fmtMoney(p.revenue)} {t('units.currency')}</span>
								<span class="ts-units">×{p.units_sold}</span>
							</span>
						</li>
					{/each}
				</ol>
			{/if}
		</section>

		<!-- Sales heatmap ──────────────────────────────── -->
		<SalesHeatmap />

		<!-- Top sellers + dead stock side-by-side ───────── -->
		<div class="cols">
			<section class="block">
				<h2>{t('analytics.topSellers')}</h2>
				{#if topSellers.length === 0}
					<p class="muted">{t('analytics.noTopSellers')}</p>
				{:else}
					<ol class="list">
						{#each topSellers as s, i (s.id)}
							<li class="row" style="--delay: {Math.min(i * 50, 400)}ms">
								<span class="row-name">{s.name}</span>
								<span class="row-meta">
									<span class="row-stock">×{s.sold_count}</span>
									<span class="row-rev">{fmtMoney(s.revenue)} {t('units.currency')}</span>
								</span>
							</li>
						{/each}
					</ol>
				{/if}
			</section>

			<section class="block">
				<h2>{t('analytics.deadStock')}</h2>
				<p class="hint">{t('analytics.deadStockHint')}</p>
				{#if deadStock.length === 0}
					<p class="muted">{t('analytics.noDeadStock')}</p>
				{:else}
					<ol class="list">
						{#each deadStock as d, i (d.id)}
							<li class="row warn" style="--delay: {Math.min(i * 50, 400)}ms">
								<span class="row-name">{d.name}</span>
								<span class="row-meta">
									<span class="row-stock">{d.current_stock} {t('units.currency') === '₾' ? '' : ''}</span>
									<span class="row-rev">{fmtMoney(d.locked_value)} {t('units.currency')}</span>
								</span>
							</li>
						{/each}
					</ol>
				{/if}
			</section>
		</div>

		<!-- Stock-out forecast ─────────────────────────── -->
		<section class="block">
			<h2>{t('analytics.stockOut')}</h2>
			<p class="hint">{t('analytics.stockOutHint')}</p>
			{#if forecast.length === 0}
				<p class="muted">{t('analytics.stockOutNoHistory')}</p>
			{:else}
				<ol class="forecast">
					{#each forecast as f, i (f.id)}
						{@const urgency =
							f.days_until_stockout <= 3
								? 'crit'
								: f.days_until_stockout <= 7
									? 'warn'
									: 'soft'}
						<li class="fc-row {urgency}" style="--delay: {Math.min(i * 50, 400)}ms">
							<span class="fc-name">{f.name}</span>
							<span class="fc-bar"
								><span
									class="fc-fill"
									style="--w: {Math.min(
										100,
										Math.max(
											8,
											(f.current_stock /
												Math.max(f.current_stock + f.sold_count, 1)) *
												100
										)
									)}%"
								></span></span
							>
							<span class="fc-meta">
								<span class="fc-stock">{f.current_stock}</span>
								<span class="fc-velocity">
									{t('analytics.stockOutVelocity', {
										v: f.velocity_per_day.toFixed(2)
									})}
								</span>
								<span class="fc-days"
									>{t('analytics.stockOutDaysLeft', {
										days: Math.max(0, Math.round(f.days_until_stockout))
									})}</span
								>
							</span>
						</li>
					{/each}
				</ol>
			{/if}
		</section>

		<!-- Low stock list ──────────────────────────────── -->
		<section class="block">
			<h2>{t('analytics.lowStock')}</h2>
			{#if lowStock.length === 0}
				<p class="muted">{t('analytics.noLowStock')}</p>
			{:else}
				<div class="chips">
					{#each lowStock as item (item.id)}
						<span class="chip" class:warn={item.current_stock <= 1}>
							<span class="chip-name">{item.name}</span>
							<span class="chip-stock">{item.current_stock}</span>
						</span>
					{/each}
				</div>
			{/if}
		</section>

		<!-- Category breakdown ──────────────────────────── -->
		<section class="block">
			<h2>{t('analytics.categories')}</h2>
			<p class="hint">{t('analytics.categoriesHint')}</p>
			{#if categories.length === 0}
				<p class="muted">{t('analytics.noCategories')}</p>
			{:else}
				<ol class="cats">
					{#each categories as c, i (c.category)}
						<li class="cat-row" style="--delay: {Math.min(i * 50, 400)}ms">
							<span class="cat-name">{c.category}</span>
							<span class="cat-bar"
								><span
									class="cat-fill"
									style="--w: {maxCategoryRevenue ? (c.revenue / maxCategoryRevenue) * 100 : 0}%"
								></span></span
							>
							<span class="cat-meta">
								<span class="cat-share"
									>{totalCategoryRevenue
										? Math.round((c.revenue / totalCategoryRevenue) * 100)
										: 0}%</span
								>
								<span class="cat-rev">{fmtMoney(c.revenue)} {t('units.currency')}</span>
							</span>
						</li>
					{/each}
				</ol>
			{/if}
		</section>

		<!-- Recent activity ─────────────────────────────── -->
		<section class="block">
			<h2>{t('analytics.activity')}</h2>
			{#if activity.length === 0}
				<p class="muted">{t('analytics.noActivity')}</p>
			{:else}
				<ul class="feed">
					{#each activity as a (a.hlc_timestamp + a.item_id)}
						<li class="feed-row" data-type={a.event_type}>
							<span class="feed-tag">{a.event_type}</span>
							<span class="feed-text">{a.summary}</span>
							<span class="feed-time">{a.hlc_timestamp}</span>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	{/if}
</div>

<style>
	.scene {
		display: flex;
		flex-direction: column;
		gap: 1.6rem;
		width: min(1100px, 94vw);
		margin: 0 auto;
		animation: rise var(--dur) var(--ease-out) both;
	}

	.head {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.kicker {
		font-size: 0.78rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--accent);
		opacity: 0.85;
	}
	.sub {
		font-family: var(--font-display);
		font-weight: 300;
		font-size: clamp(1.1rem, 2.2vw, 1.5rem);
		line-height: 1.5;
		color: var(--text-strong);
		max-width: 70ch;
		margin: 0;
	}

	.status {
		font-size: 0.9rem;
		color: var(--text-soft);
	}
	.status.err {
		color: var(--danger);
	}
	.empty {
		padding: 3rem 0;
		text-align: center;
		color: var(--text-faint);
	}
	.pulse {
		animation: pulse 1.4s ease-in-out infinite;
	}

	.kpis {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 0.75rem;
	}
	.kpi {
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		padding: 0.9rem 1rem;
		border-radius: 14px;
		background: var(--glass-bg);
		border: 1px solid var(--glass-border);
		box-shadow: var(--glass-shadow);
		backdrop-filter: blur(8px);
		animation: rise var(--dur) var(--ease-out) both;
	}
	.kpi .label {
		font-size: 0.68rem;
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.kpi .value {
		font-family: var(--font-display);
		font-size: 1.4rem;
		font-weight: 400;
		color: var(--text-strong);
		font-variant-numeric: tabular-nums;
	}
	.kpi.warn .value {
		color: var(--danger);
	}

	.block {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
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
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}
	h2 {
		font-family: var(--font-display);
		font-weight: 400;
		font-size: 1.05rem;
		color: var(--text-strong);
		margin: 0;
	}
	.hint,
	.muted {
		font-size: 0.78rem;
		color: var(--text-faint);
		margin: 0;
	}

	.bucket-switch {
		display: inline-flex;
		gap: 0.25rem;
		padding: 0.18rem;
		border-radius: 999px;
		background: rgba(20, 26, 32, 0.45);
		border: 1px solid var(--glass-border);
	}
	.bucket-switch button {
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
	.bucket-switch button:hover {
		color: var(--text);
	}
	.bucket-switch button.active {
		background: var(--accent-soft);
		color: var(--text-strong);
	}

	/* time-series list */
	.ts {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.ts-row {
		display: grid;
		grid-template-columns: 7.5rem 1fr auto;
		align-items: center;
		gap: 0.8rem;
		font-variant-numeric: tabular-nums;
	}
	.ts-label {
		font-size: 0.78rem;
		color: var(--text-soft);
	}
	.ts-bar {
		display: block;
		height: 6px;
		border-radius: 999px;
		background: rgba(160, 180, 195, 0.10);
		position: relative;
		overflow: hidden;
	}
	.ts-fill {
		position: absolute;
		inset: 0;
		width: var(--w);
		background: linear-gradient(90deg, var(--accent-soft), var(--accent));
		transition: width var(--dur-slow) var(--ease-out);
	}
	.ts-meta {
		display: inline-flex;
		gap: 0.7rem;
		font-size: 0.8rem;
		color: var(--text);
	}
	.ts-rev {
		color: var(--text-strong);
	}
	.ts-units {
		color: var(--text-faint);
	}

	/* two-column layout for top sellers / dead stock */
	.cols {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 1rem;
	}

	.list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
	}
	.row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 0.8rem;
		padding: 0.55rem 0;
		border-bottom: 1px solid rgba(160, 180, 195, 0.08);
		animation: rise var(--dur) var(--ease-out) both;
		animation-delay: var(--delay);
	}
	.row:last-child {
		border-bottom: 0;
	}
	.row-name {
		font-size: 0.92rem;
		color: var(--text);
	}
	.row-meta {
		display: inline-flex;
		gap: 0.8rem;
		font-size: 0.82rem;
		font-variant-numeric: tabular-nums;
	}
	.row-stock {
		color: var(--text-soft);
	}
	.row-rev {
		color: var(--text-strong);
	}
	.row.warn .row-name {
		color: var(--danger);
		opacity: 0.95;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem;
	}
	.chip {
		display: inline-flex;
		align-items: baseline;
		gap: 0.45rem;
		padding: 0.32rem 0.7rem;
		border-radius: 999px;
		background: rgba(20, 26, 32, 0.5);
		border: 1px solid var(--glass-border);
		font-size: 0.78rem;
		color: var(--text);
	}
	.chip-stock {
		color: var(--text-faint);
		font-variant-numeric: tabular-nums;
	}
	.chip.warn {
		border-color: rgba(220, 110, 110, 0.35);
	}
	.chip.warn .chip-stock {
		color: var(--danger);
	}

	/* stock-out forecast */
	.forecast {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
	}
	.fc-row {
		display: grid;
		grid-template-columns: 9rem 1fr auto;
		align-items: center;
		gap: 0.8rem;
		padding: 0.5rem 0;
		border-bottom: 1px solid rgba(160, 180, 195, 0.08);
		font-variant-numeric: tabular-nums;
		animation: rise var(--dur) var(--ease-out) both;
		animation-delay: var(--delay);
	}
	.fc-row:last-child {
		border-bottom: 0;
	}
	.fc-name {
		font-size: 0.9rem;
		color: var(--text);
	}
	.fc-bar {
		display: block;
		height: 6px;
		border-radius: 999px;
		background: rgba(160, 180, 195, 0.10);
		position: relative;
		overflow: hidden;
	}
	.fc-fill {
		position: absolute;
		inset: 0;
		width: var(--w);
		background: linear-gradient(90deg, var(--accent-soft), var(--accent));
		transition: width var(--dur-slow) var(--ease-out);
	}
	.fc-meta {
		display: inline-flex;
		gap: 0.7rem;
		font-size: 0.8rem;
		align-items: baseline;
	}
	.fc-stock {
		color: var(--text-soft);
	}
	.fc-velocity {
		color: var(--text-faint);
		font-size: 0.74rem;
	}
	.fc-days {
		color: var(--text-strong);
		min-width: 4.5rem;
		text-align: right;
	}
	.fc-row.crit {
		border-left: 2px solid var(--danger);
		padding-left: 0.6rem;
	}
	.fc-row.crit .fc-days {
		color: var(--danger);
	}
	.fc-row.warn .fc-days {
		color: #c4a86a;
	}
	.fc-row.soft .fc-days {
		color: var(--text);
	}

	.cats {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
	}
	.cat-row {
		display: grid;
		grid-template-columns: 10rem 1fr auto;
		align-items: center;
		gap: 0.8rem;
		font-variant-numeric: tabular-nums;
		animation: rise var(--dur) var(--ease-out) both;
		animation-delay: var(--delay);
	}
	.cat-name {
		font-size: 0.86rem;
		color: var(--text);
	}
	.cat-bar {
		display: block;
		height: 6px;
		border-radius: 999px;
		background: rgba(160, 180, 195, 0.10);
		position: relative;
		overflow: hidden;
	}
	.cat-fill {
		position: absolute;
		inset: 0;
		width: var(--w);
		background: linear-gradient(90deg, var(--accent-soft), var(--accent));
		transition: width var(--dur-slow) var(--ease-out);
	}
	.cat-meta {
		display: inline-flex;
		gap: 0.7rem;
		font-size: 0.8rem;
	}
	.cat-share {
		color: var(--text-soft);
	}
	.cat-rev {
		color: var(--text-strong);
	}

	.feed {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
	.feed-row {
		display: grid;
		grid-template-columns: 8.5rem 1fr auto;
		align-items: baseline;
		gap: 0.7rem;
		padding: 0.4rem 0;
		border-bottom: 1px solid rgba(160, 180, 195, 0.06);
		font-size: 0.82rem;
	}
	.feed-row:last-child {
		border-bottom: 0;
	}
	.feed-tag {
		font-size: 0.7rem;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--text-faint);
	}
	.feed-row[data-type='SaleRecorded'] .feed-tag {
		color: var(--accent);
	}
	.feed-row[data-type='PriceChanged'] .feed-tag {
		color: #b6c89a;
	}
	.feed-row[data-type='StockAdjusted'] .feed-tag {
		color: #c4b48a;
	}
	.feed-text {
		color: var(--text);
	}
	.feed-time {
		color: var(--text-faint);
		font-variant-numeric: tabular-nums;
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

	@media (max-width: 640px) {
		.ts-row,
		.cat-row,
		.feed-row {
			grid-template-columns: 1fr;
			gap: 0.25rem;
		}
	}
</style>