import * as vscode from 'vscode';
import * as lc from 'vscode-languageclient';

import { Config } from './config';
import { createClient } from './client';
import { isRustEditor, RustEditor } from './util';
import { DocumentSemanticsTokensSignature } from 'vscode-languageclient/lib/semanticTokens.proposed';

export class Ctx {
    private readonly onDidSendConfigurationResponseEmitter = new vscode.EventEmitter<undefined>();
    get onDidSendConfigurationResponse(): vscode.Event<undefined> {
        return this.onDidSendConfigurationResponseEmitter.event;
    }

    private constructor(
        readonly config: Config,
        private readonly extCtx: vscode.ExtensionContext,
        readonly client: lc.LanguageClient,
        readonly serverPath: string,
    ) {

    }

    static async create(
        config: Config,
        extCtx: vscode.ExtensionContext,
        serverPath: string,
        cwd: string,
    ): Promise<Ctx> {
        const client = createClient(serverPath, cwd, {
            // Workaround for https://github.com/microsoft/vscode-languageserver-node/issues/576
            ["provideDocumentSemanticTokens" as any]: async (document: vscode.TextDocument, token: vscode.CancellationToken, next: DocumentSemanticsTokensSignature) => {
                const res = await next(document, token);
                if (res === undefined) throw new Error('busy');
                return res;
            },
            // Workaround for https://github.com/rust-analyzer/rust-analyzer/issues/3924
            workspace: {
                async configuration(params, token, next) {
                    const res = await next(params, token);
                    ctx.onDidSendConfigurationResponseEmitter.fire();
                    return res;
                }
            }
        });
        const ctx = new Ctx(config, extCtx, client, serverPath);
        ctx.pushCleanup(client.start());
        await client.onReady();
        return ctx;
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

    registerCommand(name: string, factory: (ctx: Ctx) => Cmd) {
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

    pushCleanup(d: Disposable) {
        this.extCtx.subscriptions.push(d);
    }
}

export interface Disposable {
    dispose(): void;
}
export type Cmd = (...args: any[]) => unknown;
