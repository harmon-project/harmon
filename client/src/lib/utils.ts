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
