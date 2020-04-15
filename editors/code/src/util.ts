import * as lc from "vscode-languageclient";
import * as vscode from "vscode";
import * as path from 'path';
import * as fs from 'fs';
import * as util from 'util';
import { strict as nativeAssert } from "assert";

export function assert(condition: boolean, explanation: string): asserts condition {
    try {
        nativeAssert(condition, explanation);
    } catch (err) {
        log.error(`Assertion failed:`, explanation);
        throw err;
    }
}


export const log = new class {
    private enabled = true;

    setEnabled(yes: boolean): void {
        log.enabled = yes;
    }

    debug(message?: any, ...optionalParams: any[]): void {
        if (!log.enabled) return;
        // eslint-disable-next-line no-console
        console.log(message, ...optionalParams);
    }

    error(message?: any, ...optionalParams: any[]): void {
        if (!log.enabled) return;
        debugger;
        // eslint-disable-next-line no-console
        console.error(message, ...optionalParams);
    }
};

export async function sendRequestWithRetry<TParam, TRet>(
    client: lc.LanguageClient,
    reqType: lc.RequestType<TParam, TRet, unknown>,
    param: TParam,
    token?: vscode.CancellationToken,
): Promise<TRet> {
    for (const delay of [2, 4, 6, 8, 10, null]) {
        try {
            return await (token
                ? client.sendRequest(reqType, param, token)
                : client.sendRequest(reqType, param)
            );
        } catch (error) {
            if (delay === null) {
                log.error("LSP request timed out", { method: reqType.method, param, error });
                throw error;
            }

            if (error.code === lc.ErrorCodes.RequestCancelled) {
                throw error;
            }

            if (error.code !== lc.ErrorCodes.ContentModified) {
                log.error("LSP request failed", { method: reqType.method, param, error });
                throw error;
            }

            await sleep(10 * (1 << delay));
        }
    }
    throw 'unreachable';
}

export function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export type RustDocument = vscode.TextDocument & { languageId: "rust" };
export type RustEditor = vscode.TextEditor & { document: RustDocument };

export function isRustDocument(document: vscode.TextDocument): document is RustDocument {
    return document.languageId === 'rust'
        // SCM diff views have the same URI as the on-disk document but not the same content
        && document.uri.scheme !== 'git'
        && document.uri.scheme !== 'svn';
}

export function isRustEditor(editor: vscode.TextEditor): editor is RustEditor {
    return isRustDocument(editor.document);
}

export function createWorkspaceWithNewLocation(workspace: vscode.WorkspaceFolder, newLoc: vscode.Uri) {
    return {
        ...workspace,
        name: path.basename(newLoc.fsPath),
        uri: newLoc,
      };
}

// searches up the folder structure until it finds a Cargo.toml
export async function nearestParentWithCargoToml(
    workspaceRootUri: vscode.Uri,
    fileLoc: vscode.Uri,
  ): Promise<vscode.Uri | null> {
    const file_exists: (path: fs.PathLike) => Promise<boolean> = util.promisify(fs.exists);
    // check that the workspace folder already contains the "Cargo.toml"
    const workspaceRoot = workspaceRootUri.fsPath;
    // algorithm that will strip one folder at a time and check if that folder contains "Cargo.toml"
    let current = fileLoc.fsPath;
    if (fileLoc.fsPath.substring(0,workspaceRoot.length) !== workspaceRoot) {
        return null;
    }
    while (true) {
      const old = current;
      current = path.dirname(current);

      // break in case there is a bug that could result in a busy loop
      if (old === current) {
        break;
      }

      // break in case the strip folder reached the workspace root
      if (workspaceRoot === current) {
        break;
      }

      // check if "Cargo.toml" is present in the parent folder
      const cargoPath = path.join(current, 'Cargo.toml');
      if (await file_exists(cargoPath)) {
        // ghetto change the uri on Workspace folder to make vscode think it's located elsewhere
        return vscode.Uri.file(current);
      }
    }

    return null;
  }
