import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";

import * as commands from "./commands";
import { type CommandFactory, Ctx, fetchWorkspace } from "./ctx";
import * as diagnostics from "./diagnostics";
import { activateTaskProvider } from "./tasks";
import { setContextValue } from "./util";
import type { JsonProject } from "./rust_project";
import * as ra from "./lsp_ext";

const RUST_PROJECT_CONTEXT_NAME = "inRustProject";

// This API is not stable and may break in between minor releases.
export interface RustAnalyzerExtensionApi {
    readonly client?: lc.LanguageClient;

    setWorkspaces(workspaces: JsonProject[]): void;
    notifyRustAnalyzer(): Promise<void>;
}

export async function deactivate() {
    await setContextValue(RUST_PROJECT_CONTEXT_NAME, undefined);
}

export async function activate(
    context: vscode.ExtensionContext,
): Promise<RustAnalyzerExtensionApi> {
    checkConflictingExtensions();

    const ctx = new Ctx(context, createCommands(), fetchWorkspace());
    // VS Code doesn't show a notification when an extension fails to activate
    // so we do it ourselves.
    const api = await activateServer(ctx).catch((err) => {
        void vscode.window.showErrorMessage(
            `Cannot activate rust-analyzer extension: ${err.message}`,
        );
        throw err;
    });
    await setContextValue(RUST_PROJECT_CONTEXT_NAME, true);
    return api;
}

async function activateServer(ctx: Ctx): Promise<RustAnalyzerExtensionApi> {
    if (ctx.workspace.kind === "Workspace Folder") {
        ctx.pushExtCleanup(activateTaskProvider(ctx.config));
    }

    const diagnosticProvider = new diagnostics.TextDocumentProvider(ctx);
    ctx.pushExtCleanup(
        vscode.workspace.registerTextDocumentContentProvider(
            diagnostics.URI_SCHEME,
            diagnosticProvider,
        ),
    );

    const decorationProvider = new diagnostics.AnsiDecorationProvider(ctx);
    ctx.pushExtCleanup(decorationProvider);

    async function decorateVisibleEditors(document: vscode.TextDocument) {
        for (const editor of vscode.window.visibleTextEditors) {
            if (document === editor.document) {
                await decorationProvider.provideDecorations(editor);
            }
        }
    }

    vscode.workspace.onDidChangeTextDocument(
        async (event) => await decorateVisibleEditors(event.document),
        null,
        ctx.subscriptions,
    );
    vscode.workspace.onDidOpenTextDocument(decorateVisibleEditors, null, ctx.subscriptions);
    vscode.window.onDidChangeActiveTextEditor(
        async (editor) => {
            if (editor) {
                diagnosticProvider.triggerUpdate(editor.document.uri);
                await decorateVisibleEditors(editor.document);
            }
        },
        null,
        ctx.subscriptions,
    );
    vscode.window.onDidChangeVisibleTextEditors(
        async (visibleEditors) => {
            for (const editor of visibleEditors) {
                diagnosticProvider.triggerUpdate(editor.document.uri);
                await decorationProvider.provideDecorations(editor);
            }
        },
        null,
        ctx.subscriptions,
    );

    vscode.workspace.onDidChangeWorkspaceFolders(
        async (_) => ctx.onWorkspaceFolderChanges(),
        null,
        ctx.subscriptions,
    );
    vscode.workspace.onDidChangeConfiguration(
        async (_) => {
            await ctx.client?.sendNotification(lc.DidChangeConfigurationNotification.type, {
                settings: "",
            });
        },
        null,
        ctx.subscriptions,
    );
    vscode.workspace.onWillSaveTextDocument(async (event) => {
        const client = ctx.client;
        const document = event.document;
        if (document.languageId === "rust" && client) {
            // get 'rust-analyzer.autoFixDiagnostics' configuration, empty by default
            const diagnosticsToFix =
                vscode.workspace
                    .getConfiguration("rust-analyzer")
                    .get<string[]>("autoFixDiagnostics") || [];
            event.waitUntil(autoFixDiagnostics(document, diagnosticsToFix, client));
        }
    });

    await ctx.start();
    return ctx;
}

function createCommands(): Record<string, CommandFactory> {
    return {
        onEnter: {
            enabled: commands.onEnter,
            disabled: (_) => () => vscode.commands.executeCommand("default:type", { text: "\n" }),
        },
        restartServer: {
            enabled: (ctx) => async () => {
                await ctx.restart();
            },
            disabled: (ctx) => async () => {
                await ctx.start();
            },
        },
        startServer: {
            enabled: (ctx) => async () => {
                await ctx.start();
            },
            disabled: (ctx) => async () => {
                await ctx.start();
            },
        },
        stopServer: {
            enabled: (ctx) => async () => {
                // FIXME: We should re-use the client, that is ctx.deactivate() if none of the configs have changed
                await ctx.stopAndDispose();
                ctx.setServerStatus({
                    health: "stopped",
                });
            },
            disabled: (_) => async () => {},
        },

        analyzerStatus: { enabled: commands.analyzerStatus },
        memoryUsage: { enabled: commands.memoryUsage },
        shuffleCrateGraph: { enabled: commands.shuffleCrateGraph },
        reloadWorkspace: { enabled: commands.reloadWorkspace },
        rebuildProcMacros: { enabled: commands.rebuildProcMacros },
        matchingBrace: { enabled: commands.matchingBrace },
        joinLines: { enabled: commands.joinLines },
        parentModule: { enabled: commands.parentModule },
        syntaxTree: { enabled: commands.syntaxTree },
        viewHir: { enabled: commands.viewHir },
        viewMir: { enabled: commands.viewMir },
        interpretFunction: { enabled: commands.interpretFunction },
        viewFileText: { enabled: commands.viewFileText },
        viewItemTree: { enabled: commands.viewItemTree },
        viewCrateGraph: { enabled: commands.viewCrateGraph },
        viewFullCrateGraph: { enabled: commands.viewFullCrateGraph },
        expandMacro: { enabled: commands.expandMacro },
        run: { enabled: commands.run },
        copyRunCommandLine: { enabled: commands.copyRunCommandLine },
        debug: { enabled: commands.debug },
        newDebugConfig: { enabled: commands.newDebugConfig },
        openDocs: { enabled: commands.openDocs },
        openExternalDocs: { enabled: commands.openExternalDocs },
        openCargoToml: { enabled: commands.openCargoToml },
        peekTests: { enabled: commands.peekTests },
        moveItemUp: { enabled: commands.moveItemUp },
        moveItemDown: { enabled: commands.moveItemDown },
        cancelFlycheck: { enabled: commands.cancelFlycheck },
        clearFlycheck: { enabled: commands.clearFlycheck },
        runFlycheck: { enabled: commands.runFlycheck },
        ssr: { enabled: commands.ssr },
        serverVersion: { enabled: commands.serverVersion },
        viewMemoryLayout: { enabled: commands.viewMemoryLayout },
        toggleCheckOnSave: { enabled: commands.toggleCheckOnSave },
        // Internal commands which are invoked by the server.
        applyActionGroup: { enabled: commands.applyActionGroup },
        applySnippetWorkspaceEdit: { enabled: commands.applySnippetWorkspaceEditCommand },
        debugSingle: { enabled: commands.debugSingle },
        gotoLocation: { enabled: commands.gotoLocation },
        linkToCommand: { enabled: commands.linkToCommand },
        resolveCodeAction: { enabled: commands.resolveCodeAction },
        runSingle: { enabled: commands.runSingle },
        showReferences: { enabled: commands.showReferences },
        triggerParameterHints: { enabled: commands.triggerParameterHints },
        openLogs: { enabled: commands.openLogs },
        revealDependency: { enabled: commands.revealDependency },
    };
}

function checkConflictingExtensions() {
    if (vscode.extensions.getExtension("rust-lang.rust")) {
        vscode.window
            .showWarningMessage(
                `You have both the rust-analyzer (rust-lang.rust-analyzer) and Rust (rust-lang.rust) ` +
                    "plugins enabled. These are known to conflict and cause various functions of " +
                    "both plugins to not work correctly. You should disable one of them.",
                "Got it",
            )
            .then(() => {}, console.error);
    }

    if (vscode.extensions.getExtension("panicbit.cargo")) {
        vscode.window
            .showWarningMessage(
                `You have both the rust-analyzer (rust-lang.rust-analyzer) and Cargo (panicbit.cargo) plugins enabled, ` +
                    'you can disable it or set {"cargo.automaticCheck": false} in settings.json to avoid invoking cargo twice',
                "Got it",
            )
            .then(() => {}, console.error);
    }
}

async function autoFixDiagnostics(
    document: vscode.TextDocument,
    diagnosticsToFix: string[],
    client: lc.LanguageClient,
) {
    // get the diagnosis specified by the user for the current document
    const getDiagnostics = () => {
        const isInclude = (diagnostic: vscode.Diagnostic) => {
            const diagnosticCode =
                typeof diagnostic.code === "string" || typeof diagnostic.code === "number"
                    ? diagnostic.code
                    : diagnostic.code?.value || "";
            return diagnosticsToFix.includes(diagnosticCode.toString());
        };

        const diagnostics = vscode.languages.getDiagnostics(document.uri);
        return diagnostics.filter((diagnostic) => isInclude(diagnostic));
    };

    let diagnostics = getDiagnostics();

    while (diagnostics.length !== 0) {
        const currentDiagnostic = diagnostics.at(0);
        if (!currentDiagnostic) return;
        const params: lc.CodeActionParams = {
            textDocument: { uri: document.uri.toString() },
            range: currentDiagnostic.range,
            context: {
                diagnostics: [client.code2ProtocolConverter.asDiagnostic(currentDiagnostic)],
            },
        };

        const actions = await client.sendRequest(ra.codeActionForDiagnostic, params);
        const action = actions?.at(0);
        if (lc.CodeAction.is(action) && action.edit) {
            const edit = await client.protocol2CodeConverter.asWorkspaceEdit(action.edit);
            await vscode.workspace.applyEdit(edit);
        } else if (action) {
            const resolvedCodeAction = await client.sendRequest(
                lc.CodeActionResolveRequest.type,
                action,
            );
            if (resolvedCodeAction.edit) {
                const edit = await client.protocol2CodeConverter.asWorkspaceEdit(
                    resolvedCodeAction.edit,
                );
                await vscode.workspace.applyEdit(edit);
            }
        }

        // after the above `applyEdit(edit)`, source code changed, so we refresh the diagnostics
        diagnostics = getDiagnostics();
    }
}
