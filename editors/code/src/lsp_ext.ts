/**
 * This file mirrors `crates/rust-analyzer/src/lsp_ext.rs` declarations.
 */

import { TestMessage } from "vscode";
import * as lc from "vscode-languageclient";

export interface AnalyzerStatusParams {
    textDocument?: lc.TextDocumentIdentifier;
}
export const analyzerStatus = new lc.RequestType<AnalyzerStatusParams, string, void>("rust-analyzer/analyzerStatus");
export const memoryUsage = new lc.RequestType0<string, void>("rust-analyzer/memoryUsage");

export interface ServerStatusParams {
    health: "ok" | "warning" | "error";
    quiescent: boolean;
    message?: string;
}
export const serverStatus = new lc.NotificationType<ServerStatusParams>("experimental/serverStatus");

export const reloadWorkspace = new lc.RequestType0<null, void>("rust-analyzer/reloadWorkspace");

export interface SyntaxTreeParams {
    textDocument: lc.TextDocumentIdentifier;
    range: lc.Range | null;
}
export const syntaxTree = new lc.RequestType<SyntaxTreeParams, string, void>("rust-analyzer/syntaxTree");

export const viewHir = new lc.RequestType<lc.TextDocumentPositionParams, string, void>("rust-analyzer/viewHir");

export interface ViewItemTreeParams {
    textDocument: lc.TextDocumentIdentifier;
}

export const viewItemTree = new lc.RequestType<ViewItemTreeParams, string, void>("rust-analyzer/viewItemTree");

export interface ViewCrateGraphParams {
    full: boolean;
}

export const viewCrateGraph = new lc.RequestType<ViewCrateGraphParams, string, void>("rust-analyzer/viewCrateGraph");

export interface ExpandMacroParams {
    textDocument: lc.TextDocumentIdentifier;
    position: lc.Position;
}
export interface ExpandedMacro {
    name: string;
    expansion: string;
}
export const expandMacro = new lc.RequestType<ExpandMacroParams, ExpandedMacro | null, void>("rust-analyzer/expandMacro");

export interface MatchingBraceParams {
    textDocument: lc.TextDocumentIdentifier;
    positions: lc.Position[];
}
export const matchingBrace = new lc.RequestType<MatchingBraceParams, lc.Position[], void>("experimental/matchingBrace");

export const parentModule = new lc.RequestType<lc.TextDocumentPositionParams, lc.LocationLink[], void>("experimental/parentModule");

export interface JoinLinesParams {
    textDocument: lc.TextDocumentIdentifier;
    ranges: lc.Range[];
}
export const joinLines = new lc.RequestType<JoinLinesParams, lc.TextEdit[], void>("experimental/joinLines");

export const onEnter = new lc.RequestType<lc.TextDocumentPositionParams, lc.TextEdit[], void>("experimental/onEnter");

export interface RunnablesParams {
    textDocument: lc.TextDocumentIdentifier;
    position: lc.Position | null;
}

export interface Runnable {
    label: string;
    location?: lc.LocationLink;
    kind: "cargo";
    args: {
        workspaceRoot?: string;
        cargoArgs: string[];
        cargoExtraArgs: string[];
        executableArgs: string[];
        expectTest?: boolean;
        overrideCargo?: string;
    };
}
export const runnables = new lc.RequestType<RunnablesParams, Runnable[], void>("experimental/runnables");

export interface TestInfo {
    runnable: Runnable;
}

export const relatedTests = new lc.RequestType<lc.TextDocumentPositionParams, TestInfo[], void>("rust-analyzer/relatedTests");

export type InlayHint = InlayHint.TypeHint | InlayHint.ParamHint | InlayHint.ChainingHint;

export namespace InlayHint {
    export const enum Kind {
        TypeHint = "TypeHint",
        ParamHint = "ParameterHint",
        ChainingHint = "ChainingHint",
    }
    interface Common {
        range: lc.Range;
        label: string;
    }
    export type TypeHint = Common & { kind: Kind.TypeHint };
    export type ParamHint = Common & { kind: Kind.ParamHint };
    export type ChainingHint = Common & { kind: Kind.ChainingHint };
}
export interface InlayHintsParams {
    textDocument: lc.TextDocumentIdentifier;
}
export const inlayHints = new lc.RequestType<InlayHintsParams, InlayHint[], void>("rust-analyzer/inlayHints");

export interface SsrParams {
    query: string;
    parseOnly: boolean;
    textDocument: lc.TextDocumentIdentifier;
    position: lc.Position;
    selections: lc.Range[];
}
export const ssr = new lc.RequestType<SsrParams, lc.WorkspaceEdit, void>('experimental/ssr');

export interface CommandLink extends lc.Command {
    /**
     * A tooltip for the command, when represented in the UI.
     */
    tooltip?: string;
}

export interface CommandLinkGroup {
    title?: string;
    commands: CommandLink[];
}

export const openDocs = new lc.RequestType<lc.TextDocumentPositionParams, string | void, void>('experimental/externalDocs');

export interface OpenCargoTomlParams {
    textDocument: lc.TextDocumentIdentifier;
}

export const openCargoToml = new lc.RequestType<OpenCargoTomlParams, lc.Location, void>("experimental/openCargoToml");

export interface MoveItemParams {
    textDocument: lc.TextDocumentIdentifier;
    range: lc.Range;
    direction: Direction;
}

export const enum Direction {
    Up = "Up",
    Down = "Down"
}

export const moveItem = new lc.RequestType<MoveItemParams, lc.TextEdit[], void>("experimental/moveItem");

export enum RunKind {
    Run,
    Debug,
}

export interface RunTestsParams {
    include?: string[],
    exclude?: string[],
    runKind: RunKind,
};

export const runTests = new lc.RequestType<RunTestsParams, void, void>("experimental/runTests");

export const cancelTests = new lc.RequestType0<null, void>("experimenral/cancel");

export enum RunStatusUpdateKind {
    Started = 0,
    Failed = 1,
    Errored = 2,
    Passed = 3,
    Skiped = 4,
    Finish = 5,
    RawOutput = 6,
}

export type RunStatusUpdate = (Started | Failed | Errored | Passed | RawOutput | Skiped | Finish)[];

export interface Skiped {
    kind: RunStatusUpdateKind.Skiped,
    id: string,
};

export interface Finish {
    kind: RunStatusUpdateKind.Finish,
};

export interface Started {
    kind: RunStatusUpdateKind.Started,
    id: string,
};

export interface Failed {
    kind: RunStatusUpdateKind.Failed,
    id: string,
    message: TestMessage | TestMessage[],
    duration: number,
};

export interface Errored {
    kind: RunStatusUpdateKind.Errored,
    id: string,
    message: TestMessage | TestMessage[],
    duration: number,
};

export interface Passed {
    kind: RunStatusUpdateKind.Passed,
    id: string,
    duration: number,
};

export interface RawOutput {
    kind: RunStatusUpdateKind.RawOutput,
    id: string,
    message: string,
};

export const runStatus = new lc.NotificationType<RunStatusUpdate>("experimental/runStatusUpdate");