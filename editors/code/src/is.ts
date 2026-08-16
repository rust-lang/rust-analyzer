// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function boolean(value: any): value is boolean {
    return value === true || value === false;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function string(value: any): value is string {
    return typeof value === "string" || value instanceof String;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function number(value: any): value is number {
    return typeof value === "number" || value instanceof Number;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function error(value: any): value is Error {
    return value instanceof Error;
}

export function func(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    value: any,
): // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
value is Function {
    return typeof value === "function";
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function array<T>(value: any): value is T[] {
    return Array.isArray(value);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function stringArray(value: any): value is string[] {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return array(value) && (<any[]>value).every((elem) => string(elem));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function typedArray<T>(value: any, check: (value: any) => boolean): value is T[] {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return Array.isArray(value) && (<any[]>value).every(check);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function thenable<T>(value: any): value is Thenable<T> {
    return value && func(value.then);
}

export function asPromise<T>(value: Promise<T>): Promise<T>;
export function asPromise<T>(value: Thenable<T>): Promise<T>;
export function asPromise<T>(value: T): Promise<T>;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function asPromise(value: any): Promise<any> {
    if (value instanceof Promise) {
        return value;
    } else if (thenable(value)) {
        return new Promise((resolve, reject) => {
            value.then(
                (resolved) => resolve(resolved),
                (error) => reject(error),
            );
        });
    } else {
        return Promise.resolve(value);
    }
}
