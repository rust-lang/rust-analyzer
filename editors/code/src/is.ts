export function boolean(value: any): value is boolean {
	return value === true || value === false;
}

export function string(value: any): value is string {
	return typeof value === 'string' || value instanceof String;
}

export function number(value: any): value is number {
	return typeof value === 'number' || value instanceof Number;
}

export function error(value: any): value is Error {
	return value instanceof Error;
}

export function func(value: any): value is Function {
	return typeof value  === 'function';
}

export function array<T>(value: any): value is T[] {
	return Array.isArray(value);
}

export function stringArray(value: any): value is string[] {
	return array(value) && (<any[]>value).every(elem => string(elem));
}

export function typedArray<T>(value: any, check: (value: any) => boolean): value is T[] {
	return Array.isArray(value) && (<any[]>value).every(check);
}

export function thenable<T>(value: any): value is Thenable<T> {
	return value && func(value.then);
}

export function asPromise<T>(value: Promise<T>): Promise<T>;
export function asPromise<T>(value: Thenable<T>): Promise<T>;
export function asPromise<T>(value: T): Promise<T>;
export function asPromise(value: any): Promise<any> {
	if (value instanceof Promise) {
		return value;
	} else if (thenable(value)) {
		return new Promise((resolve, reject) => {
			value.then((resolved) => resolve(resolved), (error) => reject(error));
		});
	} else {
		return Promise.resolve(value);
	}
}
