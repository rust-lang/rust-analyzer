import * as path from "node:path";
import { parseArgs } from "node:util";
import * as esbuild from "esbuild";

function parseCliOptions() {
    const { values } = parseArgs({
        options: {
            minify: {
                type: "boolean",
                default: false,
            },
            sourcemap: {
                type: "boolean",
                default: false,
            },
            watch: {
                type: "boolean",
                default: false,
            },
        },
        strict: true,
    });
    return {
        shouldMinify: !!values.minify,
        shouldEmitSourceMap: !!values.sourcemap,
        isWatchMode: !!values.watch,
    };
}

const { shouldMinify, shouldEmitSourceMap, isWatchMode } = parseCliOptions();

const OUT_DIR = "./out";

function createBuildOption(entryPoints) {
    /** @type {esbuild.BuildOptions} */
    const options = {
        entryPoints,
        minify: shouldMinify,
        sourcemap: shouldEmitSourceMap ? "external" : false,
        bundle: true,
        external: ["vscode"],
        format: "cjs",
        platform: "node",
        target: "node16",
        outdir: OUT_DIR,
    };
    return options;
}

async function bundleSource(options) {
    if (!isWatchMode) {
        return esbuild.build(options);
    }

    const ctx = await esbuild.context(options);
    return ctx.watch();
}

const extensionMain = bundleSource(createBuildOption(["src/main.ts"]));
await Promise.all([extensionMain]);
