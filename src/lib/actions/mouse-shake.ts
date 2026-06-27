export interface MouseShakeOptions {
	/** Direction reversals required inside `windowMs`. */
	reversals?: number;
	/** Minimum travel before a reversal counts, filtering touchpad jitter. */
	minSegmentPx?: number;
	/** Maximum time occupied by the complete gesture. */
	windowMs?: number;
	/** Ignore further gestures for this long after a match. */
	cooldownMs?: number;
	/** A pause longer than this starts a fresh gesture. */
	maxGapMs?: number;
}

type Axis = 'x' | 'y';

interface AxisState {
	direction: -1 | 0 | 1;
	travel: number;
	reversalTimes: number[];
}

const DEFAULTS: Required<MouseShakeOptions> = {
	reversals: 4,
	minSegmentPx: 24,
	windowMs: 650,
	cooldownMs: 1400,
	maxGapMs: 240
};

/**
 * Framework-free, deterministic mouse-shake recognizer. Feed it client
 * coordinates and `event.timeStamp`; it returns true once per debounced gesture.
 */
export class MouseShakeDetector {
	readonly options: Required<MouseShakeOptions>;
	#lastPoint: { x: number; y: number; time: number } | null = null;
	#lastMatch = Number.NEGATIVE_INFINITY;
	#axes: Record<Axis, AxisState> = {
		x: { direction: 0, travel: 0, reversalTimes: [] },
		y: { direction: 0, travel: 0, reversalTimes: [] }
	};

	constructor(options: MouseShakeOptions = {}) {
		this.options = { ...DEFAULTS, ...options };
	}

	push(x: number, y: number, time: number): boolean {
		const previous = this.#lastPoint;
		this.#lastPoint = { x, y, time };
		if (!previous) return false;

		if (time - previous.time > this.options.maxGapMs || time < previous.time) {
			this.#resetAxes();
			return false;
		}
		if (time - this.#lastMatch < this.options.cooldownMs) return false;

		const dx = x - previous.x;
		const dy = y - previous.y;
		const dominant: Axis = Math.abs(dx) >= Math.abs(dy) ? 'x' : 'y';
		const delta = dominant === 'x' ? dx : dy;
		if (Math.abs(delta) < 0.5) return false;

		const matched = this.#updateAxis(dominant, delta, time);
		if (matched) {
			this.#lastMatch = time;
			this.#resetAxes();
		}
		return matched;
	}

	reset(): void {
		this.#lastPoint = null;
		this.#lastMatch = Number.NEGATIVE_INFINITY;
		this.#resetAxes();
	}

	#updateAxis(axis: Axis, delta: number, time: number): boolean {
		const state = this.#axes[axis];
		const direction: -1 | 1 = delta > 0 ? 1 : -1;
		state.reversalTimes = state.reversalTimes.filter(
			(reversalTime) => time - reversalTime <= this.options.windowMs
		);

		if (state.direction === 0) {
			state.direction = direction;
			state.travel = Math.abs(delta);
			return false;
		}

		if (state.direction === direction) {
			state.travel += Math.abs(delta);
			return false;
		}

		if (state.travel >= this.options.minSegmentPx) {
			state.reversalTimes.push(time);
		}
		state.direction = direction;
		state.travel = Math.abs(delta);
		return state.reversalTimes.length >= this.options.reversals;
	}

	#resetAxes(): void {
		for (const state of Object.values(this.#axes)) {
			state.direction = 0;
			state.travel = 0;
			state.reversalTimes = [];
		}
	}
}
