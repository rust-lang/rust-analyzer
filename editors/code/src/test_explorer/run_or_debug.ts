/* eslint-disable no-console */
import * as vscode from "vscode";
import * as os from 'os';
import * as path from 'path';
import * as fs from 'fs/promises';
import * as toolchain from "../toolchain";
import { testController } from ".";
import { spawn } from "child_process";
import { assert } from "../util";
import { createArgs, prepareEnv } from "../run";
import {  getRunnableByTestItem } from "./discover_and_update";
import { TestItemControllerHelper } from "./TestItemControllerHelper";
import { getDebugConfiguration } from "../debug";
import { raContext } from "../main";
import { StreamJsonFormatRustcOutputAnalyzer, LinesJonsFormatRustOutputAnalyzer } from "./RustcOutputAnalyzer";
import { fail } from "assert";
import { NodeKind } from "./test_model_tree";

export async function runHandler(
    request: vscode.TestRunRequest,
    token: vscode.CancellationToken
) {
    // TODO: Never run tests concurrently in client side.
    // TODO: could not run on workspace/package level, waiting for https://github.com/vadimcn/codelldb/issues/948

    const chosenItems = await getChosenTestItems(request);

    if (!chosenItems) {
        return;
    }

    const testRun = testController!.createTestRun(request);

    switch (request.profile?.kind) {
        case vscode.TestRunProfileKind.Debug:
            await debugChosenTestItems(testRun, chosenItems, token);
            return;
        case vscode.TestRunProfileKind.Run:
            await runChosenTestItems(testRun, chosenItems, token);
            return;
        case vscode.TestRunProfileKind.Coverage:
            await vscode.window.showErrorMessage("Not support Coverage yet");
            break;
        case undefined:
            await vscode.window.showErrorMessage("Never run programily, which means, only run thorugh UI");
            break;
        default:
            fail("TS does not support type narrow well in switch, never run here");
    }
}

// const workspacesRunnable: ra.Runnable = {
//     label: 'test-mod ',
//     kind: 'cargo',
//     location: {
//         targetUri: "never_used",
//         targetRange: { start: { character: 0, line: 0 }, end: { character: 0, line: 0 } },
//         targetSelectionRange: { start: { character: 0, line: 0 }, end: { character: 0, line: 0 } },
//     },
//     args: {
//         cargoExtraArgs: [],
//         cargoArgs: [
//             "test",
//             "--workspace",
//             "--lib",
//             "--bins",
//             "--tests",
//         ],
//         executableArgs: [],
//     }
// };

async function getChosenTestItems(request: vscode.TestRunRequest) {
    if (request.include === undefined) {
        await vscode.window.showWarningMessage("Sorry, for now, one and only one test item need to be picked when using Testing Explorer powered by Rust-Analyzer");
        return undefined;//workspaceRunnable;
    }

    if (request.include.length === 0) {
        await vscode.window.showWarningMessage("There is no tests to run");
        return;
    }

    if (request.include.length !== 1) {
        await vscode.window.showWarningMessage("Sorry, for now, one and only one test item need to be picked when using Testing Explorer powered by Rust-Analyzer");
        return;
    }
    // Not handle exclude for now, because we only support one test item to run anyway.

    return request.include;
}

async function debugChosenTestItems(testRun: vscode.TestRun, chosenTestItems: readonly vscode.TestItem[], token: vscode.CancellationToken) {
    if (!raContext) {
        return;
    }

    // Without `await` intentionally, because we don't want to block the UI thread.
    void vscode.window.showInformationMessage("The test item status will be updated after debug session is terminated");

    assert(chosenTestItems.length === 1, "only support 1 select test item for debugging, at least for now.");
    const chosenTestItem = chosenTestItems[0]!; // safe, because we have checked the length.
    const runnable = getRunnableByTestItem(chosenTestItem);
    const runnableOrigin = runnable.origin;

    const disposables: vscode.Disposable[] = [];

    // most of the following logic comes from vscode-java-test repo. Thanks!
    const { debugConfig, isFromLacunchJson } = await getDebugConfiguration(raContext, runnableOrigin);

    if (!debugConfig) {
        return;
    }

    if (debugConfig.type !== 'lldb') {
        await vscode.window.showInformationMessage("Sorry, for now, only [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) is supported for debugging when using Testing Explorer powered by Rust-Analyzer"
            + "You can use CodeLens to debug with [MS C++ tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools)"
        );
        return;
    }

    let outputFilePath: string | undefined;

    if (isFromLacunchJson && debugConfig["stdio"]) {
        // Without `await` intentionally, because we don't want to block the UI thread.
        void vscode.window.showInformationMessage("The test choose config from launch.json and you alredy set Stdio Redirection option. We respect it but could not analytics the output.");
    } else {
        const tmpFolderPath = await fs.mkdtemp(path.join(os.tmpdir(), 'ra-test-redirect-'));
        outputFilePath = path.join(tmpFolderPath, 'output.txt');
        debugConfig["stdio"] = [null, outputFilePath];
    }

    if (runnable.testKind === NodeKind.TestModule) {
        TestItemControllerHelper.visitTestItemTreePreOrder(testItem => {
            testRun.enqueued(testItem);
        }, chosenTestItem.children);
    } else {
        testRun.enqueued(chosenTestItem);
    }

    let debugSession: vscode.DebugSession | undefined;
    disposables.push(vscode.debug.onDidStartDebugSession((session: vscode.DebugSession) => {
        // Safe, because concurrently debugging is not allowed.
        // So the name should not be duplicated
        if (session.name === debugConfig.name) {
            debugSession = session;
        }
    }));

    const success = await vscode.debug.startDebugging(undefined, debugConfig);

    if (!success || token.isCancellationRequested) {
        dispose();
        return;
    }

    token.onCancellationRequested(async () => {
        await debugSession?.customRequest('disconnect', { restart: false });
    });

    return await new Promise<void>((resolve: () => void): void => {
        disposables.push(
            vscode.debug.onDidTerminateDebugSession(async (session: vscode.DebugSession) => {
                if (debugConfig.name === session.name) {
                    debugSession = undefined;
                    if (outputFilePath) {
                        const fileLineContents = (await fs.readFile(outputFilePath, 'utf-8'))
                            .split(/\r?\n/);
                        const outputAnalyzer = new LinesJonsFormatRustOutputAnalyzer(testRun, chosenTestItem);
                        outputAnalyzer.analyticsLines(fileLineContents);
                    }
                    dispose();
                    return resolve();
                }
            }),
        );
    });

    function dispose() {
        disposables.forEach(d => d.dispose());
        disposables.length = 0;
        testRun.end();
    }
}

// refer from playwright-vscode
/**
 * @param chosenTestItems The chosen ones of test items. The test cases which should be run should be the children of them.
 */
async function runChosenTestItems(testRun: vscode.TestRun, chosenTestItems: readonly vscode.TestItem[], token: vscode.CancellationToken) {
    assert(chosenTestItems.length === 1, "only support 1 select test item for running, at least for now.");
    const chosenTestItem = chosenTestItems[0]!; // safe, because we have checked the length.
    const runnable = getRunnableByTestItem(chosenTestItem);
    const runnableOrigin = runnable.origin;

    const args = createArgs(runnableOrigin);

    const finalArgs = args
        // Remove `--nocapture`
        // so that we could analytics the output easily and always correctly.
        // Otherwise, if the case writes into stdout, due to the parallel execution,
        // the output might be messy and it might be even impossible to analytic.
        .filter(arg => arg !== '--nocapture')
        .concat(
            // enable unstable features
            '-Z',
            'unstable-options',
        )
        // convert text output to events
        // This makes the output much easier to be analyzed,
        // and also enable us to know when which case is started(otherwise, we could only know when it is finished).
        .concat('--format=json') // Not statble, need `-Z unstable-options`
        // show exact time for test
        .concat('--report-time',) // Not statble, need `-Z unstable-options`
        // show output from succeed test
        .concat('--show-output'); // statble

    const cwd = runnableOrigin.args.workspaceRoot || ".";

    assert(finalArgs[0] === 'test', "We only support 'cargo test' command in test explorer for now!");

    // TODO: add override cargo
    // overrideCargo: runnable.args.overrideCargo;
    const cargoPath = await toolchain.cargoPath();

    if (runnable.testKind === NodeKind.TestModule) {
        TestItemControllerHelper.visitTestItemTreePreOrder(testItem => {
            testRun.enqueued(testItem);
        }, chosenTestItem.children);
    } else {
        testRun.enqueued(chosenTestItem);
    }

    // output the runned command.
    testRun.appendOutput(`${cargoPath} ${finalArgs.join(' ')}` + '\r\n');

    const outputAnalyzer = new StreamJsonFormatRustcOutputAnalyzer(testRun, chosenTestItem);

    // start process and listen to the output
    const childProcess = spawn(cargoPath, finalArgs, {
        cwd,
        stdio: ['pipe', 'pipe', 'pipe', 'pipe', 'pipe'],
        // FIXME: Should we inheritage the runnableEnv too?
        env: prepareEnv(runnableOrigin, /* config.runnableEnv */undefined),
    });
    const stdio = childProcess.stdio;
    stdio[1].on('data', data => outputAnalyzer.onStdOut(data));
    stdio[2].on('data', data => outputAnalyzer.onStdErr(data));
    childProcess.on('exit', () => outputAnalyzer.onClose());
    token.onCancellationRequested(() => {
        if (!childProcess.killed) {
            childProcess.kill();
        }
        testRun.end();
    });
}
