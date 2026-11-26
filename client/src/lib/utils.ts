export function getUserColor(publicKey: string): string {
	let hash = 0;

	for (let i = 0; i < publicKey.length; i++) {
		const char = publicKey.charCodeAt(i);
		hash = (hash << 5) - hash + char;
	}

	const hue = (hash >>> 0) % 360;

	return `hsl(${hue}, 70%, 55%)`;
}

export function clamp(value: number, min: number, max: number) {
	return Math.min(Math.max(value, min), max);
}

export function debounce(callback: Function, wait = 1000) {
	let timeout: ReturnType<typeof setTimeout>;

	return (...args: any[]) => {
		clearTimeout(timeout);
		timeout = setTimeout(() => callback(...args), wait);
	};
}

export function parse<T>(input: any): T | undefined {
	try {
		const data = String(input);
		return JSON.parse(data);
	} catch {
		return undefined;
	}
}
