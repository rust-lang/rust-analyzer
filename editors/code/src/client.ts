import * as lc from 'vscode-languageclient';
import * as vscode from 'vscode';

import { CallHierarchyFeature } from 'vscode-languageclient/lib/callHierarchy.proposed';
import { SemanticTokensFeature, DocumentSemanticsTokensSignature } from 'vscode-languageclient/lib/semanticTokens.proposed';

function toTrusted(obj: vscode.MarkedString): vscode.MarkedString {
    const md = <vscode.MarkdownString>obj;
    if (md && md.value.includes("```rust")) {
        md.isTrusted = true;
        return md;
    }
    return obj;
}

interface CommandLinkGroup {
    title?: string;
    commands: vscode.Command[];
}

function renderCommand(cmd: vscode.Command) {
    return `[${cmd.title}](command:${cmd.command}?${encodeURIComponent(JSON.stringify(cmd.arguments))} '${cmd.tooltip!}')`;
}

function renderHoverActions(actions: CommandLinkGroup[]): vscode.MarkdownString {
    const text = actions.map(group =>
        (group.title ? (group.title + " ") : "") + group.commands.map(renderCommand).join(' | ')
    ).join('___');

    const result = new vscode.MarkdownString(text);
    result.isTrusted = true;
    return result;
}

export function createClient(serverPath: string, cwd: string): lc.LanguageClient {
    // '.' Is the fallback if no folder is open
    // TODO?: Workspace folders support Uri's (eg: file://test.txt).
    // It might be a good idea to test if the uri points to a file.

    const run: lc.Executable = {
        command: serverPath,
        options: { cwd },
    };
    const serverOptions: lc.ServerOptions = {
        run,
        debug: run,
    };
    const traceOutputChannel = vscode.window.createOutputChannel(
        'Rust Analyzer Language Server Trace',
    );

    const clientOptions: lc.LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'rust' }],
        initializationOptions: vscode.workspace.getConfiguration("rust-analyzer"),
        traceOutputChannel,
        middleware: {
            // Workaround for https://github.com/microsoft/vscode-languageserver-node/issues/576
            async provideDocumentSemanticTokens(document: vscode.TextDocument, token: vscode.CancellationToken, next: DocumentSemanticsTokensSignature) {
                const res = await next(document, token);
                if (res === undefined) throw new Error('busy');
                return res;
            },
            // Workaround to support actions (trusted vscode.MarkdownString) in hovers
            // https://github.com/microsoft/vscode/issues/33577
            async provideHover(document: vscode.TextDocument, position: vscode.Position, token: vscode.CancellationToken, _next: lc.ProvideHoverSignature) {
                return client.sendRequest(lc.HoverRequest.type, client.code2ProtocolConverter.asTextDocumentPositionParams(document, position), token).then(
                    (result) => {
                        const hover = client.protocol2CodeConverter.asHover(result);
                        if (hover) {
                            hover.contents = hover.contents.map(toTrusted);
                            const actions = (<any>result).actions;
                            if (actions) {
                                hover.contents.push(renderHoverActions(actions));
                            }
                        }
                        return hover;
                    },
                    (error) => {
                        client.logFailedRequest(lc.HoverRequest.type, error);
                        return Promise.resolve(null);
                    });
            },
            async provideCodeActions(document: vscode.TextDocument, range: vscode.Range, context: vscode.CodeActionContext, token: vscode.CancellationToken, _next: lc.ProvideCodeActionsSignature) {
                const params: lc.CodeActionParams = {
                    textDocument: client.code2ProtocolConverter.asTextDocumentIdentifier(document),
                    range: client.code2ProtocolConverter.asRange(range),
                    context: client.code2ProtocolConverter.asCodeActionContext(context)
                };
                return client.sendRequest(lc.CodeActionRequest.type, params, token).then((values) => {
                    if (values === null) return undefined;
                    const result: (vscode.CodeAction | vscode.Command)[] = [];
                    const groups = new Map<string, { index: number; items: vscode.CodeAction[] }>();
                    for (const item of values) {
                        if (lc.CodeAction.is(item)) {
                            const action = client.protocol2CodeConverter.asCodeAction(item);
                            const group = actionGroup(item);
                            if (isSnippetEdit(item) || group) {
                                action.command = {
                                    command: "rust-analyzer.applySnippetWorkspaceEdit",
                                    title: "",
                                    arguments: [action.edit],
                                };
                                action.edit = undefined;
                            }

                            if (group) {
                                let entry = groups.get(group);
                                if (!entry) {
                                    entry = { index: result.length, items: [] };
                                    groups.set(group, entry);
                                    result.push(action);
                                }
                                entry.items.push(action);
                            } else {
                                result.push(action);
                            }
                        } else {
                            const command = client.protocol2CodeConverter.asCommand(item);
                            result.push(command);
                        }
                    }
                    for (const [group, { index, items }] of groups) {
                        if (items.length === 1) {
                            result[index] = items[0];
                        } else {
                            const action = new vscode.CodeAction(group);
                            action.command = {
                                command: "rust-analyzer.applyActionGroup",
                                title: "",
                                arguments: [items.map((item) => {
                                    return { label: item.title, edit: item.command!!.arguments!![0] };
                                })],
                            };
                            result[index] = action;
                        }
                    }
                    return result;
                },
                    (_error) => undefined
                );
            }

        } as any
    };

    const client = new lc.LanguageClient(
        'rust-analyzer',
        'Rust Analyzer Language Server',
        serverOptions,
        clientOptions,
    );

    // To turn on all proposed features use: client.registerProposedFeatures();
    // Here we want to enable CallHierarchyFeature and SemanticTokensFeature
    // since they are available on stable.
    // Note that while these features are stable in vscode their LSP protocol
    // implementations are still in the "proposed" category for 3.16.
    client.registerFeature(new CallHierarchyFeature(client));
    client.registerFeature(new SemanticTokensFeature(client));
    client.registerFeature(new ExperimentalFeatures());

    return client;
}

class ExperimentalFeatures implements lc.StaticFeature {
    fillClientCapabilities(capabilities: lc.ClientCapabilities): void {
        const caps: any = capabilities.experimental ?? {};
        caps.snippetTextEdit = true;
        caps.codeActionGroup = true;
        capabilities.experimental = caps;
    }
    initialize(_capabilities: lc.ServerCapabilities<any>, _documentSelector: lc.DocumentSelector | undefined): void {
    }
}

function isSnippetEdit(action: lc.CodeAction): boolean {
    const documentChanges = action.edit?.documentChanges ?? [];
    for (const edit of documentChanges) {
        if (lc.TextDocumentEdit.is(edit)) {
            if (edit.edits.some((indel) => (indel as any).insertTextFormat === lc.InsertTextFormat.Snippet)) {
                return true;
            }
        }
    }
    return false;
}

function actionGroup(action: lc.CodeAction): string | undefined {
    return (action as any).group;
}
