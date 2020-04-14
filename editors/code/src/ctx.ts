import * as vscode from 'vscode';
import * as lc from 'vscode-languageclient';

import { Config } from './config';
import { createClient } from './client';
import { isRustEditor, RustEditor } from './util';
import { StatusDisplay } from './status_display';
import { WorkDoneProgress } from 'vscode-languageclient';

export class Ctx {
    private constructor(
        readonly config: Config,
        private readonly extCtx: vscode.ExtensionContext,
        readonly client: lc.LanguageClient,
        readonly serverPath: string,
        readonly subscriptions: Disposable[],
        private readonly status: StatusDisplay,
    ) {

    }

    static async create(
        config: Config,
        extCtx: vscode.ExtensionContext,
        serverPath: string,
        cwd: vscode.WorkspaceFolder,
    ): Promise<Ctx> {
        const client = await createClient(serverPath, cwd);

        const statusDisplay = new StatusDisplay(config.checkOnSave.command);
        const res = new Ctx(config, extCtx, client, serverPath, [], statusDisplay);

        res.pushCleanup(client.start());
        await client.onReady();

        res.pushCleanup(res.status);
        if (client != null) {
            res.pushCleanup(client.onProgress(
                WorkDoneProgress.type,
                'rustAnalyzer/cargoWatcher',
                params => res.status.handleProgressNotification(params)
            ));
        }
        return res;
    }

    get activeRustEditor(): RustEditor | undefined {
        const editor = vscode.window.activeTextEditor;
        return editor && isRustEditor(editor)
            ? editor
            : undefined;
    }

    get visibleRustEditors(): RustEditor[] {
        return vscode.window.visibleTextEditors.filter(isRustEditor);
    }


    get globalState(): vscode.Memento {
        return this.extCtx.globalState;
    }

    show() {
        this.status.statusBarItem.show();
    }

    hide() {
        this.status.statusBarItem.hide();
    }

    dispose() {
        for (let d of this.subscriptions) {
            d.dispose();
        }
    }

    pushCleanup(d: Disposable) {
        this.subscriptions.push(d);
    }
}

export interface Disposable {
    dispose(): void;
}
export type Cmd = (...args: any[]) => unknown;
