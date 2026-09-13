import * as assert from "node:assert/strict";
import * as vscode from "vscode";
import { matchingBrace } from "../../src/commands";
import type { CtxInit } from "../../src/ctx";
import type { MatchingBraceParams } from "../../src/lsp_ext";
import type { Context } from ".";

export async function getTests(ctx: Context) {
    await ctx.suite("Matching brace command", (suite) => {
        async function check(
            text: string,
            outside: boolean,
            cursors: number[],
            requested: number[],
            replies: number[],
            expected: number[],
            anchor?: number,
        ) {
            const document = await vscode.workspace.openTextDocument({
                language: "rust",
                content: text,
            });
            const editor = {
                document,
                selections: cursors.map(
                    (offset) =>
                        new vscode.Selection(
                            document.positionAt(anchor ?? offset),
                            document.positionAt(offset),
                        ),
                ),
                revealRange() {},
            };
            const command = matchingBrace({
                activeRustEditor: editor,
                config: { matchingBraceJumpToOutside: outside },
                client: {
                    code2ProtocolConverter: {
                        asTextDocumentIdentifier: () => ({ uri: document.uri.toString() }),
                        asPosition: (position: vscode.Position) => position,
                    },
                    protocol2CodeConverter: {
                        asPosition: (position: vscode.Position) => position,
                    },
                    async sendRequest(_method: unknown, params: MatchingBraceParams) {
                        assert.deepEqual(
                            params.positions,
                            requested.map((offset) => document.positionAt(offset)),
                        );
                        return replies.map((offset) => document.positionAt(offset));
                    },
                },
            } as unknown as CtxInit);
            await command();
            assert.deepEqual(
                editor.selections.map((selection) => document.offsetAt(selection.active)),
                expected,
            );
            assert.deepEqual(
                editor.selections.map((selection) => document.offsetAt(selection.anchor)),
                expected.map((offset) => anchor ?? offset),
            );
        }

        suite.addTest("keeps the default cursor positions", async () => {
            await check("(())", false, [0, 2], [0, 2], [3, 1], [3, 1]);
        });
        suite.addTest("jumps outside each bracket pair", async () => {
            for (const pair of ["{}", "[]", "()", "<>", "||"]) {
                await check(pair, true, [0], [0], [1], [2]);
                await check(pair, true, [2], [1], [0], [0]);
            }
        });
        suite.addTest("returns from between consecutive closing brackets", async () => {
            await check("(())", true, [1], [1], [2], [3]);
            await check("(())", true, [3], [2], [1], [1]);
        });
        suite.addTest("keeps selection anchors", async () => {
            await check("{ x }", true, [0], [0], [4], [5], 2);
        });
        suite.addTest("handles multiple cursors across lines and Unicode text", async () => {
            await check('{\n"🦀"; []\n}', true, [0, 8], [0, 8], [11, 9], [12, 10]);
        });
        suite.addTest("does not move when the server finds no match", async () => {
            await check("x >", true, [3], [2], [2], [3]);
            await check("}", true, [0], [0], [0], [0]);
        });
        suite.addTest("jumps outside the enclosing block from its body", async () => {
            await check("{ x }", true, [2], [2], [4], [5]);
        });
    });
}
