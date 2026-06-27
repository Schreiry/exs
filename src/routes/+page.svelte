<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import VoidBackground from '$lib/components/VoidBackground.svelte';
	import WindowChrome from '$lib/components/WindowChrome.svelte';
	import ContextInput from '$lib/components/ContextInput.svelte';
	import ResultScene from '$lib/components/ResultScene.svelte';
	import AnalyticsScene from '$lib/components/AnalyticsScene.svelte';
	import SalesHeatmap from '$lib/components/SalesHeatmap.svelte';
	import { ActionIsland } from '$lib/components/actions';
	import { actionIsland, registerActions } from '$lib/actions';
	import { scene } from '$lib/scene/scene.svelte';
	import { t } from '$lib/i18n';
	import {
		aiGetStatus,
		clearContextFiles,
		exportBackup,
		forgetContextFile,
		isTauri,
		rebuildSearchIndex,
		seedDemoItems,
		selectContextFiles
	} from '$lib/tauri/commands';
	import type { ContextFileMetadata } from '$lib/types';

	let draft = $state('');
	let provider = $state<string | null>(null);
	let notice = $state('');
	let activeCommand = $state<string | null>(null);
	let attachedFiles = $state<ContextFileMetadata[]>([]);
	// During thinking, the user can type additional context. Anything they
	// type here is captured separately and offered as a "+ add context" button
	// so the current AI run isn't interrupted — it merges into the next turn.
	let injectionDraft = $state('');
	let pointerFrame = 0;
	let pointerX = 0;
	let pointerY = 0;
	let composer: { focusInput: (selectEnd?: boolean) => void } | null = $state(null);
	let historyViewport: HTMLElement | null = $state(null);

	// Parallel task lanes for the thinking state. Each lane has its own clock;
	// they advance on a staggered timer so the user sees concurrent work. The
	// lane set is derived from the current query: it includes a /module lane
	// when the user invoked one.
	type LaneState = 'idle' | 'running' | 'done';
	interface Lane {
		id: string;
		label: string;
		state: LaneState;
	}
	const baseLanes: Lane[] = [
		{ id: 'search', label: 'ლოკალური ძიება', state: 'idle' },
		{ id: 'context', label: 'კონტტექსტის შეკვრა', state: 'idle' },
		{ id: 'reason', label: 'ანალიზი და აზროვნება', state: 'idle' },
		{ id: 'compose', label: 'პასუხის ფორმირება', state: 'idle' }
	];
	let thinkingLanes = $state<Lane[]>(baseLanes.map((l) => ({ ...l })));
	let laneTimers: ReturnType<typeof setTimeout>[] = [];

	function resetThinkingLanes() {
		for (const t of laneTimers) clearTimeout(t);
		laneTimers = [];
		// Writes must be untracked so this doesn't create a write→effect→write
		// loop with the $effect below (which would Svelte flag as
		// effect_update_depth_exceeded).
		untrack(() => {
			thinkingLanes = thinkingLanes.map((l) => ({ ...l, state: 'idle' }));
		});
	}

	function startThinkingLanes(query: string) {
		resetThinkingLanes();
		// /module queries add an extra lane to surface the module's API call.
		const moduleMatch = query.match(/^\[module:(\w+)\]/);
		const lanes = moduleMatch
			? [
					...baseLanes.slice(0, 3),
					{
						id: 'module',
						label: `${moduleMatch[1]} მოდულის გამოძახობა`,
						state: 'running' as LaneState
					},
					baseLanes[3]
				]
			: baseLanes;
		// Same untrack pattern: this state is driven by setTimeouts, not by
		// any other effect, so it should not loop.
		untrack(() => {
			thinkingLanes = lanes.map((l) => ({ ...l, state: 'idle' }));
		});
		// Stagger the lane activations so the user sees four waves of activity.
		const wave = 480;
		for (let i = 0; i < lanes.length; i++) {
			laneTimers.push(
				setTimeout(() => {
					untrack(() => {
						thinkingLanes = thinkingLanes.map((l, idx) =>
							idx === i ? { ...l, state: 'running' } : l
						);
						laneTimers.push(
							setTimeout(() => {
								untrack(() => {
									thinkingLanes = thinkingLanes.map((l, idx) =>
										idx === i ? { ...l, state: 'done' } : l
									);
								});
							}, wave + 280)
						);
					});
				}, i * wave)
			);
		}
	}

	$effect(() => {
		// Read deps explicitly first so Svelte's tracker sees them.
		const mode = scene.mode;
		const query = scene.query;
		untrack(() => {
			if (mode === 'searching' && query) startThinkingLanes(query);
			else if (mode !== 'searching') resetThinkingLanes();
		});
	});

	// Apply the injection draft to the current scene query. Captures the
	// additional context as a note appended to the scene, ready to be merged
	// into the next turn via the conversation history. Keeps the running AI
	// call untouched (no cancellation) — this is parallel context, not a
	// replacement.
	function applyInjection() {
		const add = injectionDraft.trim();
		if (!add) return;
		injectionDraft = '';
		notice = `+ კონტექსტი დაემატება შემდეგ ტურნს`;
		// Persist the injection as the most recent user turn fragment so the
		// AI sees it in conversation history.
		void scene.submit(`+ ${add}`, []).finally(focusComposer);
	}

	const hasTurns = $derived(scene.turns.length > 0);
	const latestIndex = $derived(scene.turns.length - 1);

	// — Module commands. Each one is invokable via /name from the chat and also
	// registered in the action island. The handler is a placeholder until real
	// APIs are wired; the routing surface is what matters. —
	type ModuleHandler = (args: string) => void;
	const modules: Record<string, { label: string; description: string; icon: string; run: ModuleHandler }> = {
		business: {
			label: t('actions.moduleBusiness'),
			description: t('actions.moduleBusinessDescription'),
			icon: '◈',
			run: (args) => routeModule('business', args, 'ბიზნეს ანალიზი', args || 'კონკურენტები და ფასების ანალიზი')
		},
		gov: {
			label: t('actions.moduleGovernment'),
			description: t('actions.moduleGovernmentDescription'),
			icon: '⬡',
			run: (args) => routeModule('gov', args, 'საჯარო სერვისები', args || 'საქართველოს საჯარო რეესტრები')
		},
		social: {
			label: t('actions.moduleSocial'),
			description: t('actions.moduleSocialDescription'),
			icon: '◉',
			run: (args) =>
				routeModule('social', args, 'სოციალური მედია', args || 'Instagram და Facebook — კონტენტის გენერაცია')
		},
		instagram: {
			label: 'Instagram',
			description: 'Instagram-პოსტის შექმნა და განთავსება',
			icon: '◐',
			run: (args) => routeModule('instagram', args, 'Instagram', args || 'პოსტის გენერაცია')
		},
		facebook: {
			label: 'Facebook',
			description: 'Facebook-პოსტის შექმნა და განთავსება',
			icon: '◑',
			run: (args) => routeModule('facebook', args, 'Facebook', args || 'პოსტის გენერაცია')
		},
		competitors: {
			label: t('actions.moduleCompetitors'),
			description: t('actions.moduleCompetitorsDescription'),
			icon: '◭',
			run: (args) => routeModule('competitors', args, 'კონკურენტები', args || 'მსგავსი ბიზნესების ფასების თვალთვალი')
		},
		stats: {
			label: t('actions.moduleStats'),
			description: t('actions.moduleStatsDescription'),
			icon: '◮',
			run: (args) => routeModule('stats', args, 'სტატისტიკა', args || 'სექტორული მონაცემები')
		}
	};

	function routeModule(id: string, _args: string, name: string, promptHint: string) {
		activeCommand = id;
		notice = t('actions.moduleIntro', { name });
		// Hand off to the AI with the module name prepended so the assistant
		// knows which module's surface to use. The module API call itself is a
		// future integration; for now, the AI is told the module context.
		const augmented = `[module:${id}] ${promptHint}`;
		void scene.submit(augmented, attachedFiles.map((f) => f.selection_id));
		setTimeout(() => (activeCommand = null), 1200);
	}

	function focusComposer() {
		requestAnimationFrame(() => composer?.focusInput());
	}

	// — Submit handler. The chat understands four prefixes:
	//   /module …   invoke a business / social / gov / stats module
	//   @file …     reference an attached file by selection id or name
	//   #product … reference a product by id or alias
	//   !query      force "thinking" mode (deeper reasoning, slower)
	//   plain text  goes straight to the AI search path.
	function submit(q: string) {
		const query = q.trim();
		if (!query || scene.mode === 'searching') return;
		draft = '';

		const prefix = query[0];
		const body = query.slice(1).trim();

		if (prefix === '/') {
			const [raw, ...rest] = body.split(/\s+/);
			const cmd = raw.toLowerCase();
			const args = rest.join(' ').trim();
			const mod = modules[cmd];
			if (mod) {
				mod.run(args);
				return;
			}
			void scene
				.submit(
					`[command-not-found] /${cmd}. ცნობილი: ${Object.keys(modules).join(', ')}`,
					[]
				)
				.finally(focusComposer);
			return;
		}

		if (prefix === '@') {
			// Reference an attached file. Match by selection_id first, then by
			// filename (case-insensitive). The AI gets a marker so it knows the
			// file is part of the context.
			const target = body.toLowerCase();
			const match = attachedFiles.find(
				(f) =>
					f.selection_id.toLowerCase() === target ||
					f.file_name.toLowerCase() === target ||
					f.file_name.toLowerCase().includes(target)
			);
			if (!match) {
				notice = `@${target} · ფაილი კონტექსტში ვერ მოიძებნა`;
				focusComposer();
				return;
			}
			void scene
				.submit(
					`[file-ref:${match.selection_id}:${match.file_name}] ${body.slice(match.file_name.length).trim() || match.file_name}`,
					[match.selection_id]
				)
				.finally(focusComposer);
			return;
		}

		if (prefix === '#') {
			// Reference a product by id or alias. The AI is told the id; the
			// full search still happens server-side to confirm it exists.
			void scene
				.submit(`[product-ref] ${body}`, [])
				.finally(focusComposer);
			return;
		}

		if (prefix === '!') {
			// Force thinking mode. Tag the query so the AI can adjust its depth.
			void scene.submit(`[think:deep] ${body}`, []).finally(focusComposer);
			return;
		}

		const contextFileIds = attachedFiles.map((file) => file.selection_id);
		void scene.submit(query, contextFileIds).finally(focusComposer);
	}

	function removeAttachment(selectionId: string) {
		attachedFiles = attachedFiles.filter((file) => file.selection_id !== selectionId);
		if (isTauri()) void forgetContextFile(selectionId);
		focusComposer();
	}

	async function clearAttachments() {
		attachedFiles = [];
		if (isTauri()) await clearContextFiles();
	}

	async function pickAttachments() {
		const selection = await selectContextFiles();
		if (selection.cancelled) return;

		const known = new Map(
			attachedFiles.map((file) => [
				`${file.file_name}:${file.size_bytes}:${file.modified_at ?? ''}`,
				file
			])
		);
		for (const file of selection.files) {
			const key = `${file.file_name}:${file.size_bytes}:${file.modified_at ?? ''}`;
			const duplicate = known.get(key);
			if (duplicate) {
				void forgetContextFile(file.selection_id);
			} else {
				known.set(key, file);
			}
		}
		const accepted = [...known.values()];
		attachedFiles = accepted.slice(0, 10);
		for (const extra of accepted.slice(10)) {
			void forgetContextFile(extra.selection_id);
		}
		const rejected = selection.rejected.length;
		notice = rejected
			? t('files.attachedRejected', { count: selection.files.length, rejected })
			: t('files.attached', { count: attachedFiles.length });
		focusComposer();
	}

	// Pointer work is coalesced to one style update per frame.
	function onMove(event: PointerEvent) {
		pointerX = event.clientX;
		pointerY = event.clientY;
		if (pointerFrame) return;
		pointerFrame = requestAnimationFrame(() => {
			pointerFrame = 0;
			const nx = (pointerX / window.innerWidth) * 2 - 1;
			const ny = (pointerY / window.innerHeight) * 2 - 1;
			const root = document.documentElement.style;
			root.setProperty('--px', nx.toFixed(3));
			root.setProperty('--py', ny.toFixed(3));
		});
	}

	// The whole scene is the input. A printable key starts composing immediately.
// While the AI is busy, printable keys route to a parallel injection buffer
// instead of the main draft, so the user can add context without interrupting
// the running call.
	function captureKeyboard(event: KeyboardEvent) {
		if (
			event.defaultPrevented ||
			actionIsland.isOpen ||
			event.altKey ||
			event.ctrlKey ||
			event.metaKey ||
			event.isComposing
		) {
			return;
		}
		const target = event.target as HTMLElement | null;
		if (
			target?.matches('textarea, input, select, button, [contenteditable="true"]') ||
			target?.closest('[contenteditable="true"]')
		) {
			return;
		}
		if (event.key.length !== 1) return;
		event.preventDefault();
		if (scene.mode === 'searching') {
			injectionDraft += event.key;
		} else {
			draft += event.key;
			focusComposer();
		}
	}

	$effect(() => {
		const count = scene.turns.length;
		if (!count || !historyViewport) return;
		requestAnimationFrame(() => {
			historyViewport?.scrollTo({
				top: historyViewport.scrollHeight,
				behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'
			});
		});
	});

	onMount(() => {
		const unregister = registerActions(
			[
				{
					id: 'context.focus',
					label: t('actions.focus'),
					description: t('actions.focusDescription'),
					icon: '⌁',
					group: t('actions.contextGroup'),
					tone: 'accent',
					order: 10,
					dismiss: 'immediate',
					run: () => focusComposer()
				},
				{
					id: 'files.attach',
					label: t('actions.attachFiles'),
					description: t('actions.attachFilesDescription'),
					icon: '◇',
					group: t('actions.aiContextGroup'),
					tone: 'accent',
					order: 30,
					enabled: () => isTauri(),
					badge: () =>
						attachedFiles.length
							? t('files.inContext', { count: attachedFiles.length })
							: undefined,
					run: pickAttachments
				},
				{
					id: 'files.clear',
					label: t('actions.clearFiles'),
					description: t('actions.clearFilesDescription'),
					icon: '⊘',
					group: t('actions.aiContextGroup'),
					order: 40,
					visible: () => attachedFiles.length > 0,
					run: clearAttachments
				},
				{
					id: 'context.clear',
					label: t('actions.newContext'),
					description: t('actions.newContextDescription'),
					icon: '◌',
					group: t('actions.contextGroup'),
					order: 20,
					run: async () => {
						scene.clearHistory();
						draft = '';
						await clearAttachments();
						focusComposer();
					}
				},
				{
					id: 'search.rebuild',
					label: t('actions.refreshSearch'),
					description: t('actions.refreshSearchDescription'),
					icon: '⌕',
					group: t('actions.dataGroup'),
					tone: 'success',
					order: 50,
					enabled: () => isTauri(),
					run: async () => {
						const count = await rebuildSearchIndex();
						notice = t('system.indexUpdated', { count });
					}
				},
				{
					id: 'data.backup',
					label: t('actions.createBackup'),
					description: t('actions.createBackupDescription'),
					icon: '⬡',
					group: t('actions.dataGroup'),
					tone: 'neutral',
					order: 60,
					enabled: () => isTauri(),
					run: async () => {
						await exportBackup();
						notice = t('system.backupReady');
					}
				},
				// — Module commands: also discoverable in the action island. —
				{
					id: 'module.business',
					label: t('actions.moduleBusiness'),
					description: t('actions.moduleBusinessDescription'),
					icon: '◈',
					group: t('actions.businessGroup') || 'ბიზნესი',
					tone: 'accent',
					order: 70,
					run: () => modules.business.run('')
				},
				{
					id: 'module.gov',
					label: t('actions.moduleGovernment'),
					description: t('actions.moduleGovernmentDescription'),
					icon: '⬡',
					group: t('actions.businessGroup') || 'ბიზნესი',
					order: 80,
					run: () => modules.gov.run('')
				},
				{
					id: 'module.social',
					label: t('actions.moduleSocial'),
					description: t('actions.moduleSocialDescription'),
					icon: '◉',
					group: t('actions.socialGroup') || 'სოციალური',
					tone: 'accent',
					order: 90,
					run: () => modules.social.run('')
				},
				{
					id: 'module.instagram',
					label: 'Instagram',
					description: 'Instagram-პოსტის შექმნა და განთავსება',
					icon: '◐',
					group: t('actions.socialGroup') || 'სოციალური',
					order: 100,
					run: () => modules.instagram.run('')
				},
				{
					id: 'module.facebook',
					label: 'Facebook',
					description: 'Facebook-პოსტის შექმნა და განთავსება',
					icon: '◑',
					group: t('actions.socialGroup') || 'სოციალური',
					order: 110,
					run: () => modules.facebook.run('')
				},
				{
					id: 'module.competitors',
					label: t('actions.moduleCompetitors'),
					description: t('actions.moduleCompetitorsDescription'),
					icon: '◭',
					group: t('actions.businessGroup') || 'ბიზნესი',
					order: 120,
					run: () => modules.competitors.run('')
				},
				{
					id: 'module.stats',
					label: t('actions.moduleStats'),
					description: t('actions.moduleStatsDescription'),
					icon: '◮',
					group: t('actions.businessGroup') || 'ბიზნესი',
					order: 130,
					run: () => modules.stats.run('')
				},
				// — Thinking mode toggle. —
				{
					id: 'think.toggle',
					label: 'ღრმა აზროვნება',
					description: 'მომდევნო მოთხოვნა ღრმა რეჟიმში',
					icon: '!',
					group: t('actions.contextGroup'),
					order: 15,
					dismiss: 'immediate',
					run: () => {
						draft = '!';
						focusComposer();
					}
				}
			],
			{ replace: true }
		);

		// Startup stays visually immediate: backend conveniences run after first paint.
		const idle = window.setTimeout(() => {
			if (!isTauri()) {
				provider = 'mock';
				return;
			}
			void seedDemoItems().catch(() => {});
			void aiGetStatus()
				.then((status) => (provider = status.selected))
				.catch(() => {});
		}, 0);

		focusComposer();
		return () => {
			window.clearTimeout(idle);
			cancelAnimationFrame(pointerFrame);
			unregister();
		};
	});
</script>

<svelte:window onpointermove={onMove} onkeydown={captureKeyboard} />

<VoidBackground />
<WindowChrome
	labels={{
		titlebar: 'Exsul-ის ფანჯრის ზედა ზოლი',
		controls: 'ფანჯრის მართვა',
		minimize: 'ჩაკეცვა',
		maximize: 'გაშლა',
		restore: 'აღდგენა',
		close: 'დახურვა'
	}}
/>

<main class="stage" class:has-turns={hasTurns} class:writing={draft.length > 0}>
	<header class="hero">
		<span class="kicker">CONTEXTUAL INTELLIGENCE</span>
		<h1>{t('app.title')}</h1>
		<p>{t('app.tagline')}</p>
	</header>

	<section
		bind:this={historyViewport}
		class="memory"
		class:empty={!hasTurns && scene.mode !== 'searching' && scene.mode !== 'error'}
		aria-label="კონტექსტის ისტორია"
		aria-live="polite"
	>
		<div class="memory-stream">
			{#each scene.turns as turn, index (turn.id)}
				<article class="turn" class:archived={index !== latestIndex}>
					<div class="prompt">
						<span>თქვენ · {String(index + 1).padStart(2, '0')}</span>
						<p>{turn.query}</p>
					</div>
					{#if index === latestIndex}
						<ResultScene response={turn.response} />
					{:else}
						<div class="memory-summary">
							<span>{t('results.found', { count: turn.response.search.results.length })}</span>
							<p>{turn.response.answer?.text || turn.response.search.assistant_summary}</p>
						</div>
					{/if}
				</article>
			{/each}

			{#if scene.mode === 'searching'}
				<!-- Parallel-task thinking panel. The four lanes represent the
				     pipeline the AI is going through; they advance in waves so the
				     user perceives parallel work. The user can type during thinking
				     to inject context — anything they add is queued and merged into
				     the next turn. -->
				<article class="turn pending">
					<div class="prompt">
						<span>თქვენ · ახლა</span>
						<p>{scene.query}</p>
					</div>
					<div class="thinking">
						<div class="thinking-lanes" aria-hidden="true">
							{#each thinkingLanes as lane, i (lane.id)}
								<div
									class="lane"
									class:done={lane.state === 'done'}
									class:running={lane.state === 'running'}
									class:idle={lane.state === 'idle'}
									style="--i: {i}"
								>
									<span class="lane-mark">
										{#if lane.state === 'done'}◉{:else if lane.state === 'running'}<span class="lane-pulse"></span>{:else}◯{/if}
									</span>
									<span class="lane-label">{lane.label}</span>
								</div>
							{/each}
						</div>
						<p class="thinking-caption">
							<span class="thinking-dot" aria-hidden="true"></span>
							<span>{t('state.thinking')}</span>
						</p>
						{#if injectionDraft.trim()}
							<button
								type="button"
								class="injection-apply"
								onclick={applyInjection}
							>
								<span class="injection-mark">+</span>
								<span class="injection-label">კონტექსტის დამატება · {injectionDraft.trim().slice(0, 38)}{injectionDraft.trim().length > 38 ? '…' : ''}</span>
							</button>
						{/if}
					</div>
				</article>
			{:else if scene.mode === 'error'}
				<div class="scene-error" role="alert">
					<span>{t('state.error')}</span>
					<p>{scene.error}</p>
				</div>
			{/if}
		</div>
	</section>

	<section class="composer-zone" aria-label="კონტექსტის ველი">
		<ContextInput
			bind:this={composer}
			bind:value={draft}
			busy={scene.mode === 'searching'}
			attachments={attachedFiles}
			onremoveattachment={removeAttachment}
			onsubmit={submit}
		/>
	</section>

	{#if scene.mode === 'analytics'}
		<div class="results-wrap">
			<AnalyticsScene />
		</div>
	{/if}

	<footer class="scene-meta">
		<div class="provider">
			<span class="provider-dot"></span>
			AI · {provider ?? '…'}
		</div>
		{#if notice}<span class="notice">{notice}</span>{/if}
		<button type="button" onclick={() => actionIsland.open('api')}>
			<span>{t('actions.open')}</span>
			<kbd>ALT + C</kbd>
		</button>
	</footer>
</main>

<ActionIsland
	title={t('actions.title')}
	hint={t('actions.hint')}
	emptyLabel={t('actions.empty')}
	closeLabel="ქმედებების სივრცის დახურვა"
	navigationHint="↑ ↓ ნავიგაცია · Enter არჩევა · Alt+C გახსნა"
/>

<style>
	.stage {
		position: relative;
		z-index: 1;
		width: 100vw;
		height: 100vh;
		overflow: hidden;
	}

	.hero {
		position: absolute;
		z-index: 2;
		top: clamp(5.8rem, 12vh, 8.2rem);
		left: 50%;
		display: grid;
		justify-items: center;
		width: min(900px, 90vw);
		text-align: center;
		transform: translateX(-50%);
		transition:
			opacity var(--dur) var(--ease-soft),
			transform var(--dur-slow) var(--ease-out);
	}
	.kicker {
		color: var(--accent);
		font-family: var(--font-display);
		font-size: 0.65rem;
		font-weight: 650;
		letter-spacing: 0.28em;
	}
	.hero h1 {
		margin-top: 0.3rem;
		color: var(--text-strong);
		font-family: var(--font-display);
		font-size: clamp(3.5rem, 8.5vw, 7.8rem);
		font-weight: 560;
		line-height: 0.9;
		letter-spacing: 0.18em;
		text-indent: 0.18em;
		text-transform: uppercase;
		text-shadow:
			0 4px 44px rgba(138, 154, 140, 0.14),
			0 30px 80px rgba(0, 0, 0, 0.32);
	}
	.hero p {
		margin-top: 0.85rem;
		color: var(--text-soft);
		font-size: clamp(0.85rem, 1.4vw, 1.05rem);
		font-weight: 530;
		letter-spacing: 0.08em;
	}
	.has-turns .hero {
		opacity: 0;
		transform: translate(-50%, -20px) scale(0.98);
		pointer-events: none;
	}
	.writing .hero {
		opacity: 0;
	}

	.memory {
		position: absolute;
		z-index: 1;
		inset: 4.7rem 0 48%;
		padding: 1rem max(3vw, 1rem) 3rem;
		overflow: auto;
		overscroll-behavior: contain;
		mask-image: linear-gradient(transparent, black 2rem, black calc(100% - 2rem), transparent);
		transition:
			opacity var(--dur) var(--ease-soft),
			transform var(--dur) var(--ease-out),
			filter var(--dur) var(--ease-soft);
	}
	.memory.empty {
		pointer-events: none;
	}
	.memory-stream {
		display: grid;
		gap: 3rem;
		width: min(1240px, 100%);
		margin: 0 auto;
	}
	.writing .memory {
		opacity: 0.12;
		filter: blur(3px);
		transform: scale(0.975);
		pointer-events: none;
	}

	.turn {
		display: grid;
		gap: 1.6rem;
		padding: clamp(1.3rem, 2.5vw, 2.2rem);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 30px;
		background:
			linear-gradient(145deg, rgba(255, 255, 255, 0.075), transparent 44%),
			rgba(29, 42, 52, 0.36);
		box-shadow:
			inset 0 1px 0 rgba(255, 255, 255, 0.1),
			0 28px 80px rgba(1, 7, 12, 0.25);
		backdrop-filter: blur(15px);
		animation: emerge var(--dur-slow) var(--ease-out) both;
	}
	.turn.archived {
		opacity: 0.68;
	}
	.prompt {
		display: grid;
		justify-items: center;
		gap: 0.35rem;
		text-align: center;
	}
	.prompt span,
	.memory-summary span,
	.scene-error span {
		color: var(--accent);
		font-family: var(--font-display);
		font-size: 0.64rem;
		font-weight: 680;
		letter-spacing: 0.19em;
		text-transform: uppercase;
	}
	.prompt p {
		max-width: 34ch;
		color: var(--text-strong);
		font-family: var(--font-display);
		font-size: clamp(1.5rem, 3vw, 2.8rem);
		font-weight: 590;
		line-height: 1.1;
		letter-spacing: 0.025em;
		text-transform: uppercase;
		text-wrap: balance;
	}
	.memory-summary {
		display: grid;
		justify-items: center;
		gap: 0.4rem;
		text-align: center;
	}
	.memory-summary p {
		max-width: 64ch;
		overflow: hidden;
		color: var(--text);
		font-size: 1rem;
		line-height: 1.45;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	/* — Thinking panel: four parallel lanes + caption + context-injection. — */
	.thinking {
		display: grid;
		justify-items: center;
		gap: 1.1rem;
		padding-top: 0.4rem;
	}
	.thinking-lanes {
		display: grid;
		gap: 0.5rem;
		width: min(520px, 92%);
	}
	.lane {
		display: grid;
		grid-template-columns: 18px 1fr;
		align-items: center;
		gap: 0.7rem;
		padding: 0.55rem 0.8rem;
		border: 1px solid var(--glass-border);
		border-radius: 12px;
		background: var(--glass-bg);
		backdrop-filter: blur(10px);
		color: var(--text-faint);
		font-family: var(--font-display);
		font-size: 0.78rem;
		font-weight: 540;
		letter-spacing: 0.04em;
		opacity: 0;
		transform: translateY(8px);
		filter: blur(4px);
		animation: lane-in var(--dur) var(--ease-out) forwards;
		animation-delay: calc(var(--i, 0) * 60ms + 80ms);
		transition:
			color var(--dur) var(--ease-soft),
			border-color var(--dur) var(--ease-soft),
			background var(--dur) var(--ease-soft);
	}
	.lane.running {
		color: var(--text-strong);
		border-color: var(--accent-soft, rgba(138, 154, 140, 0.28));
		background: linear-gradient(90deg, rgba(138, 154, 140, 0.05), transparent 60%);
	}
	.lane.done {
		color: var(--text-soft);
		opacity: 0.72;
	}
	.lane-mark {
		display: grid;
		place-items: center;
		width: 18px;
		height: 18px;
		font-size: 0.85rem;
		color: var(--text-faint);
	}
	.lane.running .lane-mark {
		color: var(--accent);
	}
	.lane.done .lane-mark {
		color: var(--text);
	}
	.lane-pulse {
		display: block;
		width: 9px;
		height: 9px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 14px var(--accent-glow);
		animation: lane-pulse 1.05s ease-in-out infinite;
	}
	.lane-label {
		display: inline-block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.thinking-caption {
		display: inline-flex;
		align-items: center;
		gap: 0.6rem;
		color: var(--text-soft);
		font-family: var(--font-display);
		font-size: 0.74rem;
		font-weight: 600;
		letter-spacing: 0.18em;
		text-transform: uppercase;
	}
	.thinking-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 12px var(--accent-glow);
		animation: lane-pulse 1.05s ease-in-out infinite;
	}
	.injection-apply {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.95rem;
		border: 1px dashed var(--accent-soft, rgba(138, 154, 140, 0.4));
		border-radius: 999px;
		background: transparent;
		color: var(--text-strong);
		font-family: var(--font-display);
		font-size: 0.72rem;
		font-weight: 580;
		letter-spacing: 0.04em;
		cursor: pointer;
		animation: lane-in var(--dur) var(--ease-out) forwards;
		transition: border-color var(--dur-fast), background var(--dur-fast);
	}
	.injection-apply:hover {
		background: rgba(138, 154, 140, 0.08);
		border-style: solid;
	}
	.injection-mark {
		color: var(--accent);
		font-size: 1.1rem;
		line-height: 1;
	}

	.scene-error {
		display: grid;
		justify-items: center;
		gap: 0.5rem;
		padding: 1rem;
		text-align: center;
	}
	.scene-error span {
		color: var(--danger);
	}
	.scene-error p {
		max-width: 70ch;
		color: #ffb1aa;
	}

	.composer-zone {
		position: absolute;
		z-index: 3;
		top: 58%;
		left: 50%;
		transform: translate(-50%, -50%);
		transition: top var(--dur-slow) var(--ease-out);
	}
	.has-turns .composer-zone {
		top: 70%;
	}
	.writing .composer-zone {
		top: 54%;
	}

	.scene-meta {
		position: fixed;
		z-index: 4;
		right: 1rem;
		bottom: 0.85rem;
		left: 1rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		color: var(--text-faint);
		font-family: var(--font-display);
		font-size: 0.66rem;
		font-weight: 620;
		letter-spacing: 0.11em;
		text-transform: uppercase;
	}
	.provider {
		display: flex;
		align-items: center;
		gap: 0.45rem;
	}
	.provider-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 12px var(--accent);
	}
	.notice {
		color: var(--text-soft);
		animation: emerge var(--dur) var(--ease-out);
	}
	.scene-meta button {
		display: inline-flex;
		align-items: center;
		gap: 0.65rem;
		padding: 0.45rem 0.5rem 0.45rem 0.8rem;
		border: 1px solid rgba(255, 255, 255, 0.11);
		border-radius: 999px;
		background: rgba(255, 255, 255, 0.04);
		box-shadow: inset 0 1px rgba(255, 255, 255, 0.08);
		transition:
			color var(--dur-fast),
			border-color var(--dur-fast),
			background var(--dur-fast);
	}
	.scene-meta button:hover {
		color: var(--text-strong);
		border-color: rgba(174, 184, 173, 0.28);
		background: rgba(138, 154, 140, 0.07);
	}
	.scene-meta kbd {
		padding: 0.27rem 0.45rem;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 999px;
		font: inherit;
		letter-spacing: 0.05em;
	}

	@keyframes emerge {
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
	@keyframes lane-in {
		from {
			opacity: 0;
			transform: translateY(8px);
			filter: blur(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
			filter: blur(0);
		}
	}
	@keyframes lane-pulse {
		0%,
		100% {
			opacity: 0.4;
			transform: scale(0.85);
		}
		50% {
			opacity: 1;
			transform: scale(1.2);
		}
	}
	@keyframes pulse {
		0%,
		100% {
			opacity: 0.35;
			transform: scale(0.8);
		}
		50% {
			opacity: 1;
			transform: scale(1.3);
		}
	}

	@media (max-width: 700px), (max-height: 720px) {
		.hero {
			top: 5.3rem;
		}
		.hero h1 {
			font-size: clamp(3.2rem, 16vw, 5.2rem);
		}
		.memory {
			inset: 4.2rem 0 52%;
		}
		.has-turns .composer-zone {
			top: 72%;
		}
		.scene-meta {
			font-size: 0.58rem;
		}
		.notice,
		.scene-meta button > span {
			display: none;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.turn,
		.notice,
		.thinking span {
			animation: none;
		}
	}
</style>
