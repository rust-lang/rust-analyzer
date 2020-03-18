import * as vscode from 'vscode';
import * as lc from 'vscode-languageclient';

import { Config } from './config';
import { createClient } from './client';
import { isRustEditor, RustEditor } from './util';
import { PersistentState } from './persistent_state';

export class Ctx {
    private constructor(
        readonly config: Config,
        readonly state: PersistentState,
        private readonly extCtx: vscode.ExtensionContext,
        readonly client: lc.LanguageClient
    ) {

    }

    static async create(config: Config, state: PersistentState, extCtx: vscode.ExtensionContext, serverPath: string): Promise<Ctx> {
        const client = createClient(config, serverPath);
        const res = new Ctx(config, state, extCtx, client);
        res.pushCleanup(client.start());
        await client.onReady();
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

    registerCommand<T extends unknown[]>(name: string, factory: (ctx: Ctx) => Cmd<T>): void {
        const fullName = `rust-analyzer.${name}`;
        const cmd = factory(this);
        const d = vscode.commands.registerCommand(fullName, cmd);
        this.pushCleanup(d);
    }

    get globalState(): vscode.Memento {
        return this.extCtx.globalState;
    }

    get subscriptions(): Disposable[] {
        return this.extCtx.subscriptions;
    }

    pushCleanup(d: Disposable): void {
        this.extCtx.subscriptions.push(d);
    }
}

export interface Disposable {
    dispose(): void;
}

export type Cmd<T extends unknown[]> = (...args: T) => unknown;
