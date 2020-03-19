import * as vscode from 'vscode';
import * as lc from 'vscode-languageclient';
import * as ra from '../rust-analyzer-api';

import { Ctx, Cmd } from '../ctx';
import * as sourceChange from '../source_change';

export * from './analyzer_status';
export * from './matching_brace';
export * from './join_lines';
export * from './on_enter';
export * from './parent_module';
export * from './syntax_tree';
export * from './expand_macro';
export * from './runnables';
export * from './ssr';
export * from './server_version';

export function collectGarbage(ctx: Ctx): Cmd<[], Promise<null>> {
    return async () => ctx.client.sendRequest(ra.collectGarbage, null);
}

export function showReferences(ctx: Ctx): Cmd<[string, lc.Position, lc.Location[]], void> {
    return (uri, position, locations) => {
        const client = ctx.client;
        if (client) {
            vscode.commands.executeCommand(
                'editor.action.showReferences',
                vscode.Uri.parse(uri),
                client.protocol2CodeConverter.asPosition(position),
                locations.map(client.protocol2CodeConverter.asLocation),
            );
        }
    };
}

export function applySourceChange(ctx: Ctx): Cmd<[ra.SourceChange], Promise<void>> {
    return async change => {
        await sourceChange.applySourceChange(ctx, change);
    };
}

export function selectAndApplySourceChange(ctx: Ctx): Cmd<[ra.SourceChange[]], Promise<void>> {
    return async changes => {
        if (changes.length === 1) {
            await sourceChange.applySourceChange(ctx, changes[0]);
        } else if (changes.length > 0) {
            const selectedChange = await vscode.window.showQuickPick(changes);
            if (!selectedChange) return;
            await sourceChange.applySourceChange(ctx, selectedChange);
        }
    };
}
