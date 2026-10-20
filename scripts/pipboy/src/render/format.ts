import type { Severity } from "../compare/footprint.ts";

const UNITS = ["B", "KB", "MB", "GB"];

export function bytes(n: number): string {
	let value = Math.abs(n);
	let unit = 0;
	while (value >= 1024 && unit < UNITS.length - 1) {
		value /= 1024;
		unit++;
	}
	const digits = unit >= 3 ? 2 : unit >= 1 ? 1 : 0;
	return `${n < 0 ? "−" : ""}${value.toFixed(digits)} ${UNITS[unit]}`;
}

/** `+1.40 GB (+454.0%)`, or `0` when nothing changed. */
export function change(before: number, after: number, format: (n: number) => string = bytes): string {
	const delta = after - before;
	if (delta === 0) return "0";
	const sign = delta > 0 ? "+" : "−";
	const pct = before === 0 ? "new" : `${sign}${Math.abs((delta / before) * 100).toFixed(1)}%`;
	return `${sign}${format(Math.abs(delta))} (${pct})`;
}

export const count = (n: number) => String(n);

export const ICON: Record<Severity, string> = { ok: "", warn: "🟡 ", alert: "🔴 ", fail: "❌ " };

/** Inline code for untrusted text: backticks can't close the span, and pipes can't break a table. */
export function code(text: string): string {
	return `\`${text.replace(/`/g, "'").replace(/\|/g, "\\|").replace(/[\r\n]+/g, " ")}\``;
}
