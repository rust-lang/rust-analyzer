import * as vscode from 'vscode';
import * as path from "path";
import * as os from "os";
import { promises as fs } from "fs";

import * as commands from './commands';
import { activateInlayHints } from './inlay_hints';
import { activateStatusDisplay } from './status_display';
import { Ctx } from './ctx';
import { Config, NIGHTLY_TAG } from './config';
import { log, assert } from './util';
import { PersistentState } from './persistent_state';
import { fetchRelease, download } from './net';
import { spawnSync } from 'child_process';
import { activateTaskProvider } from './tasks';

let ctx: Ctx | undefined;
const ctxes: Map<string, Ctx> = new Map();


function registerCtxCommand(name: string, factory: (ctx: Ctx) =>  Cmd, fallback: Cmd | undefined, context: vscode.ExtensionContext) {
    const fullName = `rust-analyzer.${name}`;

    async function wrapped_cmd(...args: any[]): Promise<unknown> {
        if (ctx) {
            let cmd = factory(ctx);
            return await cmd(args);
        } else if (fallback) {
            return await fallback(args);
        }
        return;
    }

    const d = vscode.commands.registerCommand(fullName, wrapped_cmd);
    context.subscriptions.push(d);
}

async function whenOpeningTextDocument(doc: vscode.TextDocument, context: vscode.ExtensionContext) {
    if (!isRustDocument(doc)) {
        return;
    }

    let workspaceRoot = vscode.workspace.getWorkspaceFolder(doc.uri);
    if (!workspaceRoot) {
        return;
    }

    let cargoRoot = await nearestParentWithCargoToml(workspaceRoot.uri, doc.uri);
    if (cargoRoot == null) {
        vscode.window.showWarningMessage("Cargo.toml could not be located");
        return;
    }


    if (ctxes.has(cargoRoot.path)) {
        ctx = ctxes.get(cargoRoot.path);
        return;
    } else {
        const workspaceOnCargoRoot = createWorkspaceWithNewLocation(workspaceRoot, cargoRoot);
        const newCtx = await activate_new(workspaceOnCargoRoot, context);
        ctxes.set(cargoRoot.path, newCtx);
        ctx = newCtx;
    }

}

async function activate_new(workspaceFolder: vscode.WorkspaceFolder, context: vscode.ExtensionContext): Promise<Ctx> {
// Register a "dumb" onEnter command for the case where server fails to
    // start.
    //
    // FIXME: refactor command registration code such that commands are
    // **always** registered, even if the server does not start. Use API like
    // this perhaps?
    //
    // ```TypeScript
    // registerCommand(
    //    factory: (Ctx) => ((Ctx) => any),
    //    fallback: () => any = () => vscode.window.showErrorMessage(
    //        "rust-analyzer is not available"
    //    ),
    // )

    const config = new Config(context);
    const state = new PersistentState(context.globalState);
    const serverPath = await bootstrap(config, state);


    let ctx = await Ctx.create(config, context, serverPath, workspaceFolder);

    context.subscriptions.push(activateTaskProvider(workspaceFolder));


    activateInlayHints(ctx);

    return ctx;
}

export async function activate(context: vscode.ExtensionContext) {

    // context.subscriptions.push(defaultOnEnter);

    // Commands which invokes manually via command palette, shortcut, etc., they will attempt to find the use the RA for the specific document's project
    const register = (name: string, command: (ctx: Ctx) => Cmd) => registerCtxCommand(name, command, undefined, context);
    const registerWithFallBack = (name: string, command: (ctx: Ctx) => Cmd, fallback: Cmd) => registerCtxCommand(name, command, fallback, context);
    register('analyzerStatus', commands.analyzerStatus);
    register('collectGarbage', commands.collectGarbage);
    register('matchingBrace', commands.matchingBrace);
    register('joinLines', commands.joinLines);
    register('parentModule', commands.parentModule);
    register('syntaxTree', commands.syntaxTree);
    register('expandMacro', commands.expandMacro);
    register('run', commands.run);

    registerWithFallBack('onEnter', commands.onEnter, () => vscode.commands.executeCommand('default:type', { text: '\n' }));

    register('ssr', commands.ssr);
    register('serverVersion', commands.serverVersion);

    // Internal commands which are invoked by the server.
    register('runSingle', commands.runSingle);
    register('debugSingle', commands.debugSingle);
    register('showReferences', commands.showReferences);
    register('applySourceChange', commands.applySourceChange);
    register('selectAndApplySourceChange', commands.selectAndApplySourceChange);

    register('reload', _ => async () => {
        void vscode.window.showInformationMessage('Reloading rust-analyzer...');
        await deactivate();
        while (context.subscriptions.length > 0) {
            try {
                context.subscriptions.pop()!.dispose();
            } catch (err) {
                log.error("Dispose error:", err);
            }
        }
        await activate(context).catch(log.error);
    });
    // Reloading is inspired by @DanTup maneuver: https://github.com/microsoft/vscode/issues/45774#issuecomment-373423895

    vscode.workspace.onDidOpenTextDocument(doc => whenOpeningTextDocument(doc, context), null, context.subscriptions);
    vscode.workspace.textDocuments.forEach(doc => whenOpeningTextDocument(doc, context));

    function changeConfig() {
        for (let ctx of ctxes.values()) {
            ctx.client?.sendNotification('workspace/didChangeConfiguration', { settings: "" });
        }
    }

    vscode.workspace.onDidChangeConfiguration(
        _ => changeConfig,
        null,
        context.subscriptions,
    );

}

export async function deactivate() {
    await ctx?.client.stop();
    ctx = undefined;
}

async function bootstrap(config: Config, state: PersistentState): Promise<string> {
    await fs.mkdir(config.globalStoragePath, { recursive: true });

    await bootstrapExtension(config, state);
    const path = await bootstrapServer(config, state);

    return path;
}

async function bootstrapExtension(config: Config, state: PersistentState): Promise<void> {
    if (config.package.releaseTag === null) return;
    if (config.channel === "stable") {
        if (config.package.releaseTag === NIGHTLY_TAG) {
            void vscode.window.showWarningMessage(
                `You are running a nightly version of rust-analyzer extension. ` +
                `To switch to stable, uninstall the extension and re-install it from the marketplace`
            );
        }
        return;
    };

    const lastCheck = state.lastCheck;
    const now = Date.now();

    const anHour = 60 * 60 * 1000;
    const shouldDownloadNightly = state.releaseId === undefined || (now - (lastCheck ?? 0)) > anHour;

    if (!shouldDownloadNightly) return;

    const release = await fetchRelease("nightly").catch((e) => {
        log.error(e);
        if (state.releaseId === undefined) { // Show error only for the initial download
            vscode.window.showErrorMessage(`Failed to download rust-analyzer nightly ${e}`);
        }
        return undefined;
    });
    if (release === undefined || release.id === state.releaseId) return;

    const userResponse = await vscode.window.showInformationMessage(
        "New version of rust-analyzer (nightly) is available (requires reload).",
        "Update"
    );
    if (userResponse !== "Update") return;

    const artifact = release.assets.find(artifact => artifact.name === "rust-analyzer.vsix");
    assert(!!artifact, `Bad release: ${JSON.stringify(release)}`);

    const dest = path.join(config.globalStoragePath, "rust-analyzer.vsix");
    await download(artifact.browser_download_url, dest, "Downloading rust-analyzer extension");

    await vscode.commands.executeCommand("workbench.extensions.installExtension", vscode.Uri.file(dest));
    await fs.unlink(dest);

    await state.updateReleaseId(release.id);
    await state.updateLastCheck(now);
    await vscode.commands.executeCommand("workbench.action.reloadWindow");
}

async function bootstrapServer(config: Config, state: PersistentState): Promise<string> {
    const path = await getServer(config, state);
    if (!path) {
        throw new Error(
            "Rust Analyzer Language Server is not available. " +
            "Please, ensure its [proper installation](https://rust-analyzer.github.io/manual.html#installation)."
        );
    }

    log.debug("Using server binary at", path);

    const res = spawnSync(path, ["--version"], { encoding: 'utf8' });
    log.debug("Checked binary availability via --version", res);
    log.debug(res, "--version output:", res.output);
    if (res.status !== 0) {
        throw new Error(`Failed to execute ${path} --version`);
    }

    return path;
}

async function getServer(config: Config, state: PersistentState): Promise<string | undefined> {
    const explicitPath = process.env.__RA_LSP_SERVER_DEBUG ?? config.serverPath;
    if (explicitPath) {
        if (explicitPath.startsWith("~/")) {
            return os.homedir() + explicitPath.slice("~".length);
        }
        return explicitPath;
    };
    if (config.package.releaseTag === null) return "rust-analyzer";

    let binaryName: string | undefined = undefined;
    if (process.arch === "x64" || process.arch === "ia32") {
        if (process.platform === "linux") binaryName = "rust-analyzer-linux";
        if (process.platform === "darwin") binaryName = "rust-analyzer-mac";
        if (process.platform === "win32") binaryName = "rust-analyzer-windows.exe";
    }
    if (binaryName === undefined) {
        vscode.window.showErrorMessage(
            "Unfortunately we don't ship binaries for your platform yet. " +
            "You need to manually clone rust-analyzer repository and " +
            "run `cargo xtask install --server` to build the language server from sources. " +
            "If you feel that your platform should be supported, please create an issue " +
            "about that [here](https://github.com/rust-analyzer/rust-analyzer/issues) and we " +
            "will consider it."
        );
        return undefined;
    }

    const dest = path.join(config.globalStoragePath, binaryName);
    const exists = await fs.stat(dest).then(() => true, () => false);
    if (!exists) {
        await state.updateServerVersion(undefined);
    }

    if (state.serverVersion === config.package.version) return dest;

    if (config.askBeforeDownload) {
        const userResponse = await vscode.window.showInformationMessage(
            `Language server version ${config.package.version} for rust-analyzer is not installed.`,
            "Download now"
        );
        if (userResponse !== "Download now") return dest;
    }

    const release = await fetchRelease(config.package.releaseTag);
    const artifact = release.assets.find(artifact => artifact.name === binaryName);
    assert(!!artifact, `Bad release: ${JSON.stringify(release)}`);

    await download(artifact.browser_download_url, dest, "Downloading rust-analyzer server", { mode: 0o755 });
    await state.updateServerVersion(config.package.version);
    return dest;
}
