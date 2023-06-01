import * as vscode from "vscode";
import { assert, assertNever } from "../util";
import { getTestItemByTestLikeNode, getTestModelByTestItem } from "./discover_and_update";
import {
    type CargoPackageNode,
    DummyRootNode,
    NodeKind,
    type TargetNode,
    type TestModuleNode,
    type TestNode,
    getPackageNodeOfTestModelNode,
} from "./test_model_tree";
import { sep } from 'node:path';

/**
 * When running tests, rust would run built target one by one and output something like:
 *
 * "Running unittests src\lib.rs (target\debug\deps\regex-6eb576da3e025f5d.exe)"
 */
class SuiteContext {
    private static sepInRegexString = sep === '\\' ? '\\\\' : sep;

    private static relativePathCaptureGroupName = 'relativePath';

    private static normalizedTargetCaptureGroupName = 'normalizedTargetName';
    /**
     * Match the relative path of the target file, and the normalized target name
     *
     * @example "Running unittests src\\lib.rs (target\\debug\\deps\\hashbrown-3547e1bc587fc63a.exe)"
     * // when target is lin/bin in Windows, the output is as above
     * // we want to get "src\\lib.rs" and "hashbrown"
     */
    private static targetPattern = new RegExp(`Running (?:unittests )?(?<${SuiteContext.relativePathCaptureGroupName}>.*?) \(.*${SuiteContext.sepInRegexString}(?<${SuiteContext.normalizedTargetCaptureGroupName}>.*?)-.*?\)`);

    /**
     * .e.g, 'src/lib.rs', seprator is os-sensitive
     */
    targetRelativePath: string;

    /**
     * normarlized target name, '-' is relaced by '_'
     *
     * please refer https://www.reddit.com/r/rust/comments/8sezkm/where_are_the_rules_for_creating_valid_rust
     */
    normalizedTargetName: string;

    private constructor(relativePath: string, normalizedTargetName: string) {
        this.normalizedTargetName = normalizedTargetName;
        this.targetRelativePath = relativePath;
    }

    public static tryParse(line:string) {
        const match = this.targetPattern.exec(line);

        if (!match) {
            return undefined;
        }

        const targetRelativePath = match.groups?.[SuiteContext.relativePathCaptureGroupName]!;
        const normalizedTargetName = match.groups?.[SuiteContext.normalizedTargetCaptureGroupName]!;

        return new SuiteContext(
            targetRelativePath,
            normalizedTargetName,
        );
    }
}

// why replace: refer https://code.visualstudio.com/api/extension-guides/testing#test-output
function normalizeOutputDataForVSCodeTestOutput(data: any): string {
    return data.toString().replace(/\r\n/g, '\n').replace(/\n/g, '\r\n');
}

class TestItemLocator {
    private readonly _testModel: CargoPackageNode | TargetNode | TestModuleNode | TestNode;

    // We only allow one test case to be runned
    constructor(chosenRunnedTestItem: vscode.TestItem) {
        const node = getTestModelByTestItem(chosenRunnedTestItem);

        assert(node.kind === NodeKind.Test
            || node.kind === NodeKind.TestModule
            || node.kind === NodeKind.Target
            || node.kind === NodeKind.CargoPackage,
            "does not support workspace level, until we allow try to guess the target"
        );

        this._testModel = node;
    }

    /**
     * @param path This is the path which is shown on the output of test result, like mod1::mod2::mod3::test1
     */
    findTestItemByRustcOutputCasePath(suiteContext: SuiteContext, path: string): vscode.TestItem | undefined {
        const {
            normalizedTargetName,
            targetRelativePath,
        } = suiteContext;
        // const workspaceRootNode = getWorkspaceNodeOfTestModelNode(this._testModel);
        let targetNode = tryGetTargetNodeOfTestModelNode(this._testModel);

        if (!targetNode) {
            const packageNode = getPackageNodeOfTestModelNode(this._testModel);

            const targetCandidates =
                // workspaceRootNode.members
                // .flatMap(packageNode => Array.from(packageNode.targets))
                Array.from(packageNode.targets)
                .filter(target =>
                    normalizeTargetName(target.name) === normalizedTargetName
                    && target.srcPath.fsPath.includes(targetRelativePath)
                );

            assert(targetCandidates.length === 1, "should find one and only one target node, but they might have same name and relative path, although it should be really rare");
            // REVIEW: What should we do if we found 2 or more candidates?
            targetNode = targetCandidates[0]!; // safe, we have checked the length
        }

        const testNode = DummyRootNode.instance.findTestLikeNodeUnderTarget(
            targetNode,
            NodeKind.Test,
            path.split('::')
        );

        const candidate = getTestItemByTestLikeNode(testNode);

        return candidate;

        function tryGetTargetNodeOfTestModelNode(testModel: TestModuleNode | TargetNode | TestNode | CargoPackageNode) {
            if (testModel.kind === NodeKind.CargoPackage) return undefined;
            while (testModel.kind !== NodeKind.Target) {
                testModel = testModel.parent;
            }
            return testModel;
        }

    }
}

function normalizeTargetName(packageName: string) {
    return packageName.replace(/-/g, '_');
}

/**
 * This analyzer assumes `--show-output` option is enabled for rustc.(which is unstable yet, so `-Z unstable-options` must also be enabled)
 */
class JsonFormatRustcOutputAnalyzer {
    private _isTestStarted = false;

    private _testItemLocator: TestItemLocator;

    private _suiteContext: SuiteContext | undefined;

    protected _testRun: vscode.TestRun;

    constructor(
         testRun: vscode.TestRun,
        testItem: vscode.TestItem,
    ) {
        this._testRun = testRun;
        this._testItemLocator = new TestItemLocator(testItem);
    }

    protected handleNewLine(line: string) {
        const isJsonOutput = line.startsWith('{');

        if (!isJsonOutput) {
            this._testRun.appendOutput(line + '\r\n'); // must be 'CRLF', refer https://code.visualstudio.com/api/extension-guides/testing#test-output
            const newSuiteContextCandidate = SuiteContext.tryParse(line);
            if (newSuiteContextCandidate) {
                this._suiteContext = newSuiteContextCandidate;
            }
        } else {
            const event = TestEvents.parse(line);
            this.analyticsEvent(event);
        }
    }

    private analyticsEvent(testEvent: TestEvents) {
        switch (testEvent.type) {
            case 'test':
                this.analyticsTestEvent(testEvent);
                break;
            case 'suite':
                this.analyticsSuiteEvent(testEvent);
                break;
            default:
                assertNever(testEvent);
        }
    }

    private analyticsTestEvent(testEvent: RustcTestCaseEvent) {
        assert(this._isTestStarted);
        assert(!!this._suiteContext, "Test must belong to a suite");
        const testItem = this._testItemLocator.findTestItemByRustcOutputCasePath(this._suiteContext, testEvent.name);

        assert(!!testItem);

        switch (testEvent.event) {
            case 'started':
                this._testRun.started(testItem);
                break;
            case 'failed':
                this._testRun.failed(testItem, testEvent.stdout ? [new vscode.TestMessage(testEvent.stdout)] : [], fromSecondsToMilliseconds(testEvent.exec_time));
                break;
            case 'ignored':
                this._testRun.skipped(testItem);
                break;
            case 'ok':
                this._testRun.passed(testItem, fromSecondsToMilliseconds(testEvent.exec_time));
                break;
            default:
                break;
        }
    }

    private analyticsSuiteEvent(testEvent: RustcSuiteEvent) {
        switch (testEvent.event) {
            case 'started':
                assert(this._isTestStarted === false);
                this._isTestStarted = true;
                break;
            case 'failed':
                assert(this._isTestStarted === true);
                this._isTestStarted = false;
                break;
            case 'ok':
                assert(this._isTestStarted === true);
                this._isTestStarted = false;
                break;
            default:
                assertNever(testEvent);
        }
    }
}

export class StreamJsonFormatRustcOutputAnalyzer extends JsonFormatRustcOutputAnalyzer {
    constructor(
        testRun: vscode.TestRun,
        testItem: vscode.TestItem,
    ) {
        super(testRun,testItem);
    }

    public onStdErr(data: any) {
        // Some messages in the output are logged as stderr
        // like
        // "Finished test [unoptimized + debuginfo] target(s) in 0.07s"
        // "Running unittests src\lib.rs (target\debug\deps\hashbrown-3547e1bc587fc63a.exe)"
        this.handleStreamData(data);
    }

    public onClose() {
        this._testRun.end();
    }

    public onStdOut(data: any) {
        this.handleStreamData(data);
    }

    private handleStreamData(data: any) {
        // It seems like the data will be end with a line breaking. Is this promised?
        const normalizedData = normalizeOutputDataForVSCodeTestOutput(data);

        const lines = normalizedData.split("\r\n");

        lines.forEach(line => {
            this.handleNewLine(line);
        });
    }
}

export class LinesJonsFormatRustOutputAnalyzer extends JsonFormatRustcOutputAnalyzer {
    constructor(
        testRun: vscode.TestRun,
        testItem: vscode.TestItem,
    ) {
        super(testRun,testItem);
    }

    public analyticsLines(lines:string[]) {
        lines.forEach(line => {
            this.handleNewLine(line);
        });
        this._testRun.end();
     }
}

type RustcSuiteEvent = RustcSuiteStartEvent | RustcSuiteEndEvent;
type TestEvents = RustcTestCaseEvent | RustcSuiteEvent;

namespace TestEvents {
    export function parse(str: string): TestEvents {
        const result = JSON.parse(str);
        assert(result.type === 'test' || result.type === 'suite');
        return result as TestEvents;
    }
}

interface RustcTestCaseEvent {
    type: "test";
    event: "started"|"ignored"| "failed" | "ok";
    name: string;
    stdout?: string;
    exec_time?: number;
}

interface RustcSuiteStartEvent{
    type: "suite";
    event: "started";
}

interface RustcSuiteEndEvent{
    type: "suite";
    event: "failed" | "ok";
    name: string;
    passed: number;
    failed: number;
    ignored: number;
    measured: number;
    filtered_out: number;
    exec_time?: number;
}

/**
 * The time from rustc is seconds, but vscode accepts milliseconds
 */
function fromSecondsToMilliseconds(seconds: number | undefined) {
    return seconds === undefined ? undefined : seconds * 1000;
}
