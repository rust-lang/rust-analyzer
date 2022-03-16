import * as vscode from 'vscode';
import * as lc from 'vscode-languageclient';
import * as ra from './lsp_ext';
import * as tasks from './tasks';

import { Ctx } from './ctx';
import { makeDebugConfig } from './debug';
import { Config, RunnableEnvCfg } from './config';

const enableRelease = "Enable --release.";
const disableRelease = "Disable --release.";

function quickPickButtons(ctx: Ctx, activeRunnable?: string) {
    const toggleRelease = ctx.runnableRelease ?
        { iconPath: new vscode.ThemeIcon("star-full"), tooltip: disableRelease } :
        { iconPath: new vscode.ThemeIcon("star"), tooltip: enableRelease };
    const saveConfig = { iconPath: new vscode.ThemeIcon("save"), tooltip: "Save as a launch.json configurtation." };

    const res = [toggleRelease];
    if (!(activeRunnable ?? "").startsWith('cargo')) {
        res.push(saveConfig);
    }
    return [toggleRelease, saveConfig];
}

export async function selectRunnable(ctx: Ctx, prevRunnable?: RunnableQuickPick, debuggeeOnly = false, showButtons: boolean = true): Promise<RunnableQuickPick | undefined> {
    const editor = ctx.activeRustEditor;
    const client = ctx.client;
    if (!editor || !client) return;

    const textDocument: lc.TextDocumentIdentifier = {
        uri: editor.document.uri.toString(),
    };

    const runnables = await client.sendRequest(ra.runnables, {
        textDocument,
        position: client.code2ProtocolConverter.asPosition(
            editor.selection.active,
        ),
    });
    const items: RunnableQuickPick[] = [];
    if (prevRunnable) {
        items.push(prevRunnable);
    }
    for (const r of runnables) {
        if (
            prevRunnable &&
            JSON.stringify(prevRunnable.runnable) === JSON.stringify(r)
        ) {
            continue;
        }

        if (debuggeeOnly && (r.label.startsWith('doctest') || r.label.startsWith('cargo'))) {
            continue;
        }
        items.push(new RunnableQuickPick(r));
    }

    if (items.length === 0) {
        // it is the debug case, run always has at least 'cargo check ...'
        // see crates\rust-analyzer\src\main_loop\handlers.rs, handle_runnables
        await vscode.window.showErrorMessage("There's no debug target!");
        return;
    }

    return await new Promise((resolve) => {
        const disposables: vscode.Disposable[] = [];
        const close = (result?: RunnableQuickPick) => {
            resolve(result);
            disposables.forEach(d => d.dispose());
        };

        const quickPick = vscode.window.createQuickPick<RunnableQuickPick>();
        quickPick.items = items;
        quickPick.title = "Select Runnable";
        if (showButtons) {
            quickPick.buttons = quickPickButtons(ctx, quickPick.value);
        }
        disposables.push(
            quickPick.onDidHide(() => close()),
            quickPick.onDidAccept(() => close(quickPick.selectedItems[0])),
            quickPick.onDidTriggerButton(async (button) => {
                switch (button.tooltip) {
                    case enableRelease:
                    case disableRelease:
                        ctx.runnableRelease = !ctx.runnableRelease;
                        if (showButtons) {
                            quickPick.buttons = quickPickButtons(ctx, quickPick.value);
                        }
                        break;
                    default:
                        await makeDebugConfig(ctx, quickPick.activeItems[0].runnable);
                        close();
                        break;
                }
            }),
            quickPick.onDidChangeActive((_active) => {
                if (showButtons) {
                    quickPick.buttons = quickPickButtons(ctx, quickPick.value);
                }
            }),
            quickPick
        );
        quickPick.show();
    });
}

export class RunnableQuickPick implements vscode.QuickPickItem {
    public label: string;
    public description?: string | undefined;
    public detail?: string | undefined;
    public picked?: boolean | undefined;

    constructor(public runnable: ra.Runnable) {
        this.label = runnable.label;
    }
}

export function prepareEnv(runnable: ra.Runnable, runnableEnvCfg: RunnableEnvCfg): Record<string, string> {
    const env: Record<string, string> = { "RUST_BACKTRACE": "short" };

    if (runnable.args.expectTest) {
        env["UPDATE_EXPECT"] = "1";
    }

    Object.assign(env, process.env as { [key: string]: string });

    if (runnableEnvCfg) {
        if (Array.isArray(runnableEnvCfg)) {
            for (const it of runnableEnvCfg) {
                if (!it.mask || new RegExp(it.mask).test(runnable.label)) {
                    Object.assign(env, it.env);
                }
            }
        } else {
            Object.assign(env, runnableEnvCfg);
        }
    }

    return env;
}

export async function createTask(runnable: ra.Runnable, config: Config, release: boolean): Promise<vscode.Task> {
    if (runnable.kind !== "cargo") {
        // rust-analyzer supports only one kind, "cargo"
        // do not use tasks.TASK_TYPE here, these are completely different meanings.

        throw `Unexpected runnable kind: ${runnable.kind}`;
    }

    const args = createArgs(runnable, release);

    const definition: tasks.CargoTaskDefinition = {
        type: tasks.TASK_TYPE,
        command: args[0], // run, test, etc...
        args: args.slice(1),
        cwd: runnable.args.workspaceRoot || ".",
        env: prepareEnv(runnable, config.runnableEnv),
        overrideCargo: runnable.args.overrideCargo,
    };

    // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
    const target = vscode.workspace.workspaceFolders![0]; // safe, see main activate()
    const cargoTask = await tasks.buildCargoTask(target, definition, runnable.label, args, config.cargoRunner, true);

    cargoTask.presentationOptions.clear = true;
    // Sadly, this doesn't prevent focus stealing if the terminal is currently
    // hidden, and will become revealed due to task exucution.
    cargoTask.presentationOptions.focus = false;

    return cargoTask;
}

export function createArgs(runnable: ra.Runnable, release: boolean): string[] {
    const args = [...runnable.args.cargoArgs]; // should be a copy!
    if (release) {
        args.push("--release");
    }
    if (runnable.args.cargoExtraArgs) {
        args.push(...runnable.args.cargoExtraArgs); // Append user-specified cargo options.
    }
    if (runnable.args.executableArgs.length > 0) {
        args.push('--', ...runnable.args.executableArgs);
    }
    return args;
}
