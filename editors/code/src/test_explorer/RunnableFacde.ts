import * as vscode from "vscode";
import type * as ra from "../lsp_ext";
import type * as lc from "vscode-languageclient";
import { assert, assertNever } from "../util";
import { TargetKind, NodeKind, type TestLikeNodeKind, type TestLocation } from "./test_model_tree";

/**
 * A wrapper of `ra.Runnable` to provide typed/cached information rather than string.
 *
 * An important asumption is the format of `label` is
 *   - if test, "test test::path". Because test always has name, so the later part could not be empty.
 *   - if test module, "test-mod test::path". Attention, the later part could be empty if it's the root module of a target.
 */
export class RunnableFacde {
    public readonly origin: ra.Runnable;

    constructor(runnable: ra.Runnable) {
        this.origin = runnable;
    }

    public toTestLocation(): TestLocation {
        return {
            uri: this.uri,
            range: new vscode.Range(
                this.origin.location!.targetSelectionRange.start.line,
                this.origin.location!.targetSelectionRange.start.character,
                this.origin.location!.targetSelectionRange.end.line,
                this.origin.location!.targetSelectionRange.end.character,
            )
        };
    }

    private _workspaceRoot?: string;

    get workspaceRoot(): string {
        if (this._workspaceRoot) return this._workspaceRoot;

        const workspaceRoot = this.origin.args.workspaceRoot;

        assert(!!workspaceRoot);

        return this._workspaceRoot = workspaceRoot;
    }

    private _testKind?: TestLikeNodeKind;

    get testKind(): TestLikeNodeKind {
        if (this._testKind)  return this._testKind;
        const testKindString = this.origin.label.split(' ')[0];

        switch (testKindString) {
            case 'test':
                return this._testKind = NodeKind.Test;
            case 'test-mod':
                return this._testKind = NodeKind.TestModule;
            default:
                throw new Error("What could it be?");
        }
    }

    get targetName(): string {
        switch (this.targetKind) {
            case TargetKind.Binary:
                assert(!!this.binaryTestFileName);
                return this.binaryTestFileName;
            case TargetKind.IntegrationTest:
                assert(!!this.integrationTestFileName);
                return this.integrationTestFileName;
            case TargetKind.Library:
                return this.packageName;
            default:
                assertNever(this.targetKind);
        }
    }

    get testPaths(): string[] {
        const testModulePath = this.origin.label.split(' ')[1]!;
        return testModulePath.split('::');
    }

    get testOrSuiteName(): string {
        const candidateName = this.testPaths[this.testPaths.length - 1];
        // It should be safe,
        // - if it's a test, this is its name
        // - if it's a test module, this is the name of module
        return candidateName!;
    }

    private _targetKind?: TargetKind;

    get targetKind(): TargetKind {
        if (this._targetKind) return this._targetKind;

        switch (true) {
            case this.origin.args.cargoArgs.includes("--lib"):
                return this._targetKind = TargetKind.Library;
            case this.origin.args.cargoArgs.includes("--test"):
                return this._targetKind = TargetKind.IntegrationTest;
            case this.origin.args.cargoArgs.includes("--bin"):
                return this._targetKind = TargetKind.Binary;
            default:
                throw new Error("Packge shold not be target level");
        }
    }

    private _packageName?: string;

    get packageName(): string {
        if (this._packageName) return this._packageName;

        const packageQualifiedNameIndex = this.origin.args.cargoArgs.findIndex(arg => arg === "--package") + 1;

        // The format of `packageQualifiedName` is `name:version`, like `hello:1.2.3`
        const packageQualifiedName = this.origin.args.cargoArgs[packageQualifiedNameIndex];

        assert(!!packageQualifiedName, "There should be a value for '--package' in runnable");

        return this._packageName = packageQualifiedName.split(':')[0]!;
    }

    private _integrationTestFileName?: string | null;

    /**
     * Only have value if `targetKind` is `TargetKind.IntegrationTest`
     */
    get integrationTestFileName(): string | null {

        if (this._integrationTestFileName !== undefined) return this._integrationTestFileName;

        const integrationTestFileNameIndex = this.origin.args.cargoArgs.findIndex(arg => arg === "--test") + 1;

        if (integrationTestFileNameIndex === 0) {
            this._integrationTestFileName = null;
        } else {
            this._integrationTestFileName = this.origin.args.cargoArgs[integrationTestFileNameIndex];
            assert(typeof this._integrationTestFileName === "string","There should be a value for '--test' in runnable");
        }

        return this._integrationTestFileName;
    }

    private _binaryTestFileName?: string | null;

    /**
     * Only have value if `targetKind` is `TargetKind.Binary`
     */
    get binaryTestFileName(): string | null {
        if (this._binaryTestFileName !== undefined) return this._binaryTestFileName;

        const integrationTestFileNameIndex = this.origin.args.cargoArgs.findIndex(arg => arg === "--bin") + 1;

        if (integrationTestFileNameIndex === 0) {
            this._binaryTestFileName = null;
        } else {
            this._binaryTestFileName = this.origin.args.cargoArgs[integrationTestFileNameIndex];
            assert(typeof this._binaryTestFileName === "string","There should be a value for '--bin' in runnable");
        }

        return this._binaryTestFileName;
    }

    private _uri?: vscode.Uri;

    get uri(): vscode.Uri {
        if (this._uri) return this._uri;

        assert(!!this.origin.location?.targetUri, "Need to investigate why targetUri is undefined");

        return this._uri = vscode.Uri.parse(this.origin.location.targetUri);
    }

    static sortByLabel(a: RunnableFacde, b: RunnableFacde): number {
        return a.origin.label.localeCompare(b.origin.label);
    }

    /**
     * Whether the runnable is a declaration module like "mod xxx;"
     */
    get isTestModuleDeclarationRunnable() {
        assert(this.testKind === NodeKind.TestModule, "Only compare definition for test module.");

        return !this.isTestModuleFileDefinitionRunnable
            // filter out module with items
            // Not accurate. But who will write `mode xxx { ... }` in one line?
            && this.origin.location?.targetRange.end.line === this.origin.location?.targetSelectionRange.end.line;
    }

    /**
     * whether the runnable is a definition module like "mod xxx { ... }"
     */
    get isTestModuleWithItemsRunnable() {
        assert(this.testKind === NodeKind.TestModule, "Only compare definition for test module.");

        return !this.isTestModuleFileDefinitionRunnable
            && !this.isTestModuleDeclarationRunnable;
    }

    /**
     * Whether the runnable is a file definition module.
     */
    get isTestModuleFileDefinitionRunnable() {
        const runnable = this.origin;

        assert(this.testKind === NodeKind.TestModule, "Only compare definition for test module.");

        assert(!!runnable.location, "Should always have location");
        
        return isRangeValueEqual(
            runnable.location.targetRange,
            runnable.location.targetSelectionRange,
        );

        function isRangeValueEqual(a: lc.Range, b: lc.Range) {
            return isPositiionValueEqual(a.start, b.start)
                && isPositiionValueEqual(a.end, b.end);
        }

        function isPositiionValueEqual(a: lc.Position, b: lc.Position) {
            return a.line === b.line
                && a.character === b.character;
        }
    }
}
