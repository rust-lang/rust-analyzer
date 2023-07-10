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
const OUT_WEBVIEW_DIR = path.resolve(OUT_DIR, "webview");

/** @type {esbuild.BuildOptions} */
const BASE_OPTIONS = {
    minify: shouldMinify,
    sourcemap: shouldEmitSourceMap ? "external" : false,
    bundle: true,
};

function createBuildOption(entryPoints) {
    /** @type {esbuild.BuildOptions} */
    const options = {
        ...BASE_OPTIONS,
        entryPoints,
        external: ["vscode"],
        format: "cjs",
        platform: "node",
        target: "node16",
        outdir: OUT_DIR,
    };
    return options;
}

function createBuildOptionForWebView(entryPoints) {
    /** @type {esbuild.BuildOptions} */
    const options = {
        ...BASE_OPTIONS,
        entryPoints,
        format: "esm",
        platform: "browser",
        // VSCode v1.78 (Electron 22) uses Chromium 108.
        // https://code.visualstudio.com/updates/v1_78
        target: "chrome108",
        outdir: OUT_WEBVIEW_DIR,
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

await Promise.all([
    bundleSource(createBuildOption(["src/main.ts"])),
    bundleSource(
        createBuildOptionForWebView([
            "src/webview/show_crate_graph.ts",
            "src/webview/show_crate_graph.css",
        ]),
    ),
    bundleSource(
        createBuildOptionForWebView([
            "src/webview/view_memory_layout.ts",
            "src/webview/view_memory_layout.css",
        ]),
    ),
]);
