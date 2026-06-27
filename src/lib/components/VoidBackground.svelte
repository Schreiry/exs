<script lang="ts">
	// Layer 0/1 — глубокий шейдер-градиент (CSS) + дрейфующие частицы в 3 слоях
	// глубины (canvas). Параллакс читается из --px/--py. Производительность:
	// частицы ограничены, rAF, DPR ≤ 1.5, авто-снижение при низком FPS,
	// уважение prefers-reduced-motion.

	let canvas: HTMLCanvasElement | null = $state(null);

	type P = { x: number; y: number; vx: number; vy: number; r: number; depth: number; a: number };

	$effect(() => {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
		let w = 0;
		let h = 0;
		let particles: P[] = [];
		let raf = 0;
		let effort = 1; // 1 = full, lowered automatically if FPS drops

		const resize = () => {
			w = window.innerWidth;
			h = window.innerHeight;
			canvas!.width = Math.floor(w * dpr);
			canvas!.height = Math.floor(h * dpr);
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
			const target = Math.round(Math.min(36, (w * h) / 34000) * effort);
			particles = Array.from({ length: target }, makeParticle);
		};

		function makeParticle(): P {
			const depth = Math.random(); // 0 far … 1 near
			return {
				x: Math.random() * w,
				y: Math.random() * h,
				vx: (Math.random() - 0.5) * 0.12 * (0.4 + depth),
				vy: (Math.random() - 0.5) * 0.12 * (0.4 + depth),
				r: 0.6 + depth * 1.8,
				depth,
				a: 0.04 + depth * 0.12
			};
		}

		const draw = () => {
			ctx.clearRect(0, 0, w, h);
			const px = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--px')) || 0;
			const py = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--py')) || 0;
			for (const p of particles) {
				if (!reduce) {
					p.x += p.vx;
					p.y += p.vy;
					if (p.x < -20) p.x = w + 20;
					if (p.x > w + 20) p.x = -20;
					if (p.y < -20) p.y = h + 20;
					if (p.y > h + 20) p.y = -20;
				}
				// nearer particles parallax more
				const ox = px * 28 * p.depth;
				const oy = py * 28 * p.depth;
				ctx.beginPath();
				ctx.arc(p.x + ox, p.y + oy, p.r, 0, Math.PI * 2);
				const green = 220 + Math.round(p.depth * 25);
				const blue = 224 + Math.round(p.depth * 26);
				ctx.fillStyle = `rgba(170, ${green}, ${blue}, ${p.a * 1.35})`;
				ctx.fill();
			}
		};

		let last = performance.now();
		let slowFrames = 0;
		const loop = (now: number) => {
			const dt = now - last;
			last = now;
			if (dt > 34 && effort > 0.5) {
				// sustained <30fps → drop effort once
				slowFrames++;
				if (slowFrames > 40) {
					effort = 0.5;
					slowFrames = 0;
					resize();
				}
			} else if (slowFrames > 0) {
				slowFrames--;
			}
			draw();
			raf = requestAnimationFrame(loop);
		};

		resize();
		window.addEventListener('resize', resize);
		if (reduce) {
			draw();
		} else {
			raf = requestAnimationFrame(loop);
		}

		return () => {
			cancelAnimationFrame(raf);
			window.removeEventListener('resize', resize);
		};
	});
</script>

<div class="void" aria-hidden="true">
	<div class="gradient"></div>
	<canvas bind:this={canvas} class="particles"></canvas>
	<div class="vignette"></div>
</div>

<style>
	.void {
		position: fixed;
		inset: 0;
		z-index: 0;
		overflow: hidden;
		background: var(--void-0);
	}
	/* A mineral aurora: expensive graphite, not a dead black backdrop. */
	.gradient {
		position: absolute;
		inset: -14%;
		background:
			radial-gradient(42% 38% at 18% 12%, rgba(116, 183, 255, 0.2), transparent 74%),
			radial-gradient(38% 40% at 82% 24%, rgba(167, 132, 255, 0.16), transparent 72%),
			radial-gradient(45% 42% at 68% 84%, rgba(77, 225, 204, 0.17), transparent 76%),
			radial-gradient(64% 56% at 50% 42%, var(--void-3) 0%, var(--void-2) 34%, var(--void-1) 62%, var(--void-0) 100%);
		transform: translate3d(calc(var(--px) * -14px), calc(var(--py) * -14px), 0);
		animation: wake var(--dur-slow) var(--ease-out) both;
	}
	.gradient::after {
		content: '';
		position: absolute;
		inset: 12%;
		background:
			linear-gradient(118deg, transparent 28%, rgba(255, 255, 255, 0.055) 46%, transparent 60%),
			repeating-linear-gradient(
				112deg,
				transparent 0,
				transparent 110px,
				rgba(197, 231, 238, 0.022) 111px,
				transparent 113px
			);
		filter: blur(1px);
		opacity: 0.75;
	}
	/* Layer 1: particle field. */
	.particles {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}
	.vignette {
		position: absolute;
		inset: 0;
		pointer-events: none;
		background:
			linear-gradient(180deg, rgba(4, 9, 14, 0.28), transparent 16%, transparent 80%, rgba(4, 9, 14, 0.26)),
			radial-gradient(80% 78% at 50% 44%, transparent 58%, rgba(2, 8, 13, 0.36) 100%);
	}
	@keyframes wake {
		from {
			opacity: 0;
			transform: scale(1.04);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}
</style>
