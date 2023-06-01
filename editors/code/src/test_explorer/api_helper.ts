import type * as vscode from 'vscode';
import { raContext } from '../main';
import * as ra from "../lsp_ext";
import * as lc from "vscode-languageclient";
import { assert } from 'console';

/**
 * A simplified definition request.
 *
 * Should only be used to get the definition of module declaration.
 *
 * And therefore, there will only be one location.
 */
const moduleDefinitionRequest = new lc.RequestType<lc.TextDocumentPositionParams, lc.LocationLink[] | null, void>('textDocument/definition');

export abstract class RaApiHelper {
    static async getTestRunnablesInFile(uri: vscode.Uri) {
        const client = raContext?.client;
        if (!client) {
            return null;
        }

        const testInfos = await client.sendRequest(ra.testRunnablesInFile, {
            textDocument: lc.TextDocumentIdentifier.create(uri.toString()),
        });

        return testInfos;
    }

    static async parentModue(uri: vscode.Uri): Promise<lc.LocationLink[] | null> {
        const client = raContext?.client;
        if (!client) {
            return null;
        }
        const documentUriString = uri.toString();
        assert(lc.DocumentUri.is(documentUriString));

        const locations = await client.sendRequest(ra.parentModule, {
            textDocument: lc.TextDocumentIdentifier.create(documentUriString),
            position: lc.Position.create(0, 0),
        });
        return locations;
    }

    static async moduleDefinition(locationLink: lc.LocationLink): Promise<lc.LocationLink[] | null> {
        const client = raContext?.client;
        if (!client) {
            return null;
        }
        const position = locationLink.targetSelectionRange.start;

        assert(lc.DocumentUri.is(locationLink.targetUri));

        const location = await client.sendRequest(moduleDefinitionRequest, {
            textDocument: lc.TextDocumentIdentifier.create(locationLink.targetUri),
            position: lc.Position.create(position.line, position.character),
        });
        return location;
    }

    /**
     *
     * @returns cargo workspaces with depdencies. One RA instance could support multi different workspaces.
     */
    static async cargoWorkspaces() {
        const client = raContext?.client;
        if (!client) {
            return null;
        }
        const cargoWorkspaces = await client.sendRequest(ra.cargoWorkspaces);

        return cargoWorkspaces;
    }
}
