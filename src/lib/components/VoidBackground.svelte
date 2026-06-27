<script lang="ts">
	// Layer 0/1/2 — three parallax layers that compose the void.
	//   Layer 0 (back-most): CSS shader aurora, --px * 6px, slow drift
	//   Layer 1 (mid):      canvas particle field, --px * 18px
	//   Layer 2 (front):    canvas particle field, --px * 36px, fewer + brighter
	// Each layer reacts to the same --px/--py CSS vars but at different magnitudes,
	// producing the "looking into depth" sensation. Honors prefers-reduced-motion.
	// DPR cap, FPS-driven effort scaling, and a reduced particle count for weak GPUs.

	let back: HTMLCanvasElement | null = $state(null);
	let front: HTMLCanvasElement | null = $state(null);

	type P = { x: number; y: number; vx: number; vy: number; r: number; a: number };

	$effect(() => {
		if (!back || !front) return;

		const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		const dpr = Math.min(window.devicePixelRatio || 1, 1.5);

		let w = 0;
		let h = 0;
		let backParticles: P[] = [];
		let frontParticles: P[] = [];
		let raf1 = 0;
		let raf2 = 0;
		let effort = 1;

		const seed = (n: number): P => ({
			x: Math.random(),
			y: Math.random(),
			vx: (Math.random() - 0.5) * 0.00006,
			vy: (Math.random() - 0.5) * 0.00006,
			r: 0.4 + Math.random() * 0.9,
			a: 0.04 + Math.random() * 0.12
		});

		const resize = () => {
			w = window.innerWidth;
			h = window.innerHeight;
			for (const c of [back, front]) {
				if (!c) continue;
				c.width = Math.floor(w * dpr);
				c.height = Math.floor(h * dpr);
				c.getContext('2d')?.setTransform(dpr, 0, 0, dpr, 0, 0);
			}
			const midCount = Math.round(Math.min(28, (w * h) / 42000) * effort);
			const foreCount = Math.round(Math.min(14, (w * h) / 90000) * effort);
			backParticles = Array.from({ length: midCount }, seed);
			frontParticles = Array.from({ length: foreCount }, seed);
		};

		const drawLayer = (
			canvas: HTMLCanvasElement,
			particles: P[],
			magnitude: number,
			baseR: number,
			alpha: number
		) => {
			const ctx = canvas.getContext('2d');
			if (!ctx) return;
			ctx.clearRect(0, 0, w, h);
			const root = document.documentElement.style;
			const px = parseFloat(root.getPropertyValue('--px')) || 0;
			const py = parseFloat(root.getPropertyValue('--py')) || 0;
			for (const p of particles) {
				if (!reduce) {
					p.x += p.vx;
					p.y += p.vy;
					if (p.x < -0.02) p.x = 1.02;
					if (p.x > 1.02) p.x = -0.02;
					if (p.y < -0.02) p.y = 1.02;
					if (p.y > 1.02) p.y = -0.02;
				}
				const ox = px * magnitude;
				const oy = py * magnitude;
				ctx.beginPath();
				ctx.arc(p.x * w + ox, p.y * h + oy, baseR + p.r * baseR, 0, Math.PI * 2);
				ctx.fillStyle = `rgba(220, 218, 213, ${p.a * alpha})`;
				ctx.fill();
			}
		};

		let last = performance.now();
		let slow = 0;
		const loop = (now: number) => {
			const dt = now - last;
			last = now;
			if (dt > 34 && effort > 0.5) {
				slow++;
				if (slow > 40) {
					effort = 0.5;
					slow = 0;
					resize();
				}
			} else if (slow > 0) slow--;
			drawLayer(back!, backParticles, 18, 1.4, 0.55);
			raf1 = requestAnimationFrame(loop);
		};

		const loopFront = (now: number) => {
			drawLayer(front!, frontParticles, 36, 2.0, 0.95);
			raf2 = requestAnimationFrame(loopFront);
		};

		resize();
		window.addEventListener('resize', resize);
		if (reduce) {
			drawLayer(back, backParticles, 18, 1.4, 0.55);
			drawLayer(front, frontParticles, 36, 2.0, 0.95);
		} else {
			raf1 = requestAnimationFrame(loop);
			raf2 = requestAnimationFrame(loopFront);
		}

		return () => {
			cancelAnimationFrame(raf1);
			cancelAnimationFrame(raf2);
			window.removeEventListener('resize', resize);
		};
	});
</script>

<div class="void" aria-hidden="true">
	<!-- Layer 0: shader aurora. Drifts on its own clock AND follows pointer slightly. -->
	<div class="aurora">
		<div class="aurora-a"></div>
		<div class="aurora-b"></div>
		<div class="aurora-c"></div>
	</div>
	<!-- Layer 1: mid particles -->
	<canvas bind:this={back} class="layer mid"></canvas>
	<!-- Layer 2: front particles (closer = larger, brighter) -->
	<canvas bind:this={front} class="layer fore"></canvas>
	<!-- Vignette brings depth back to center -->
	<div class="vignette"></div>
	<!-- Grain (optional, very subtle) -->
	<div class="grain"></div>
</div>

<style>
	.void {
		position: fixed;
		inset: 0;
		z-index: 0;
		overflow: hidden;
		background: var(--void-0);
	}

	/* Layer 0 — shader aurora. Three blurred radial gradients, drifting in opposite
	   directions + a slow pointer parallax. Reads as a soft graphite sky, not a black backdrop. */
	.aurora {
		position: absolute;
		inset: -16%;
		transform: translate3d(calc(var(--px) * -6px), calc(var(--py) * -6px), 0);
		will-change: transform;
	}
	.aurora-a,
	.aurora-b,
	.aurora-c {
		position: absolute;
		inset: 0;
		filter: blur(60px);
		opacity: 0.55;
		mix-blend-mode: screen;
		will-change: transform;
	}
	.aurora-a {
		background: radial-gradient(50% 46% at 22% 18%, rgba(138, 154, 140, 0.34), transparent 70%);
		animation: drift-a 32s var(--ease-soft) infinite alternate;
	}
	.aurora-b {
		background: radial-gradient(46% 44% at 78% 64%, rgba(166, 158, 144, 0.22), transparent 72%);
		animation: drift-b 41s var(--ease-soft) infinite alternate;
	}
	.aurora-c {
		background: radial-gradient(60% 50% at 50% 100%, rgba(58, 64, 60, 0.4), transparent 80%);
	}
	@keyframes drift-a {
		from {
			transform: translate3d(-3%, -2%, 0) scale(1.02);
		}
		to {
			transform: translate3d(4%, 3%, 0) scale(1.08);
		}
	}
	@keyframes drift-b {
		from {
			transform: translate3d(4%, -2%, 0) scale(1.06);
		}
		to {
			transform: translate3d(-5%, 4%, 0) scale(1);
		}
	}

	/* Particle layers: pointer parallax lives in the canvas JS via --px/--py. */
	.layer {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}
	.layer.mid {
		opacity: 0.85;
	}
	.layer.fore {
		opacity: 1;
	}

	/* Vignette brings focus back to the centre — the user's eye rests on the void. */
	.vignette {
		position: absolute;
		inset: 0;
		pointer-events: none;
		background:
			radial-gradient(78% 74% at 50% 46%, transparent 56%, rgba(0, 0, 0, 0.42) 100%),
			linear-gradient(180deg, rgba(0, 0, 0, 0.18), transparent 12%, transparent 78%, rgba(0, 0, 0, 0.22));
	}

	/* Subtle film grain. Catches light, sells the depth. ~3% opacity. */
	.grain {
		position: absolute;
		inset: 0;
		pointer-events: none;
		opacity: 0.035;
		mix-blend-mode: overlay;
		background-image: url("data:image/svg+xml;utf8,<svg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/></filter><rect width='100%25' height='100%25' filter='url(%23n)'/></svg>");
	}
	@media (prefers-reduced-motion: reduce) {
		.aurora-a,
		.aurora-b {
			animation: none;
		}
	}
</style>