export function required(name: string): string {
	const value = process.env[name];
	if (!value) throw new Error(`${name} must be configured`);
	return value;
}

export function endpoint(): string {
	const value = required("RIVET_ENDPOINT");
	if (
		!value.startsWith("https://") &&
		!value.startsWith("http://127.0.0.1:") &&
		!value.startsWith("http://localhost:")
	) {
		throw new Error(
			"RIVET_ENDPOINT must use HTTPS except for a local Engine",
		);
	}
	return value;
}
