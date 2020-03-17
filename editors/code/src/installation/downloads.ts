import fetch, { Response } from "node-fetch";
import { AbortSignal as IAbortSignal } from "node-fetch/externals";
import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import * as stream from "stream";
import * as util from "util";
import { log, assert, once } from "../util";
import { ArtifactReleaseInfo } from "./interfaces";

const pipeline = util.promisify(stream.pipeline);

/**
 * Downloads file from `url` and stores it at `destFilePath` with `destFilePermissions`.
 * `onProgress` callback is called on recieveing each chunk of bytes
 * to track the progress of downloading, it gets the already read and total
 * amount of bytes to read as its parameters.
 */
async function downloadFile(
    url: string,
    destFilePath: fs.PathLike,
    destFilePermissions: number,
    ct: vscode.CancellationToken,
    onProgress: (readBytes: number, totalBytes: number) => void
): Promise<boolean> {
    let res: Response;
    try {
        res = await fetch(url, { signal: new AbortSignal(ct) });
    } catch (err) {
        if (!isAbortError(err)) throw err;
        log.debug(`Canceled download to ${destFilePath} during the inital fetch`);
        return false;
    }

    if (!res.ok) {
        log.error("Error", res.status, "while downloading file from", url);
        log.error({ body: await res.text(), headers: res.headers });

        throw new Error(`Got response ${res.status} when trying to download a file.`);
    }

    const totalBytes = Number(res.headers.get('content-length'));
    assert(!Number.isNaN(totalBytes), "Sanity check of content-length protocol");

    log.debug("Downloading file of", totalBytes, "bytes size from", url, "to", destFilePath);

    let readBytes = 0;
    res.body.on("data", (chunk: Buffer) => {
        readBytes += chunk.length;
        onProgress(readBytes, totalBytes);
    });

    const destFileStream = fs.createWriteStream(destFilePath, { mode: destFilePermissions });

    try {
        await pipeline(res.body, destFileStream);
    } catch (err) {
        if (!isAbortError(err)) throw err;

        assert(ct.isCancellationRequested, "cancellation must've caused the AbortError");

        log.debug(`Download was canceled, removing probably corrupted "${destFilePath}"...`);

        await fs.promises.unlink(destFilePath);

        return false;
    }

    return new Promise<boolean>(resolve => {
        destFileStream.on("close", () => resolve(true));
        destFileStream.destroy();

        // Details on workaround: https://github.com/rust-analyzer/rust-analyzer/pull/3092#discussion_r378191131
        // Issue at nodejs repo: https://github.com/nodejs/node/issues/31776
    });
}

function isAbortError(suspect: unknown): suspect is Error {
    return suspect instanceof Error && suspect.name === "AbortError";
}

/**
 * Downloads artifact from given `downloadUrl`.
 * Creates `installationDir` if it is not yet created and puts the artifact under
 * `artifactFileName`.
 * Displays info about the download progress in an info message printing the name
 * of the artifact as `displayName`.
 */
export async function downloadArtifactWithProgressUi(
    { downloadUrl, releaseName }: ArtifactReleaseInfo,
    artifactFileName: string,
    installationDir: string,
    displayName: string,
    ct?: vscode.CancellationToken
): Promise<boolean> {
    await fs.promises.mkdir(installationDir).catch(err => assert(
        err?.code === "EEXIST",
        `Couldn't create directory "${installationDir}" to download ` +
        `${artifactFileName} artifact: ${err?.message}`
    ));

    const installationPath = path.join(installationDir, artifactFileName);

    return await vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            cancellable: true,
            title: `Downloading rust-analyzer ${displayName} (${releaseName})`
        },
        async (progress, uiCt) => {
            let lastPrecentage = 0;
            const downloadCt = ct ? anyCt(ct, uiCt) : uiCt;
            const filePermissions = 0o755; // (rwx, r_x, r_x)
            return await downloadFile(downloadUrl, installationPath, filePermissions, downloadCt, (readBytes, totalBytes) => {
                const newPercentage = (readBytes / totalBytes) * 100;
                progress.report({
                    message: newPercentage.toFixed(0) + "%",
                    increment: newPercentage - lastPrecentage
                });

                lastPrecentage = newPercentage;
            });
        }
    );
}

function anyCt(a: vscode.CancellationToken, b: vscode.CancellationToken): vscode.CancellationToken {
    const cancellation = new vscode.EventEmitter();
    const cancel = once(() => cancellation.fire());

    a.onCancellationRequested(cancel);
    b.onCancellationRequested(cancel);

    return {
        get isCancellationRequested() { return a.isCancellationRequested || b.isCancellationRequested; },
        onCancellationRequested: cancellation.event
    };
}


type Listener = (this: AbortSignal, event: any) => unknown;

/**
 * Ad hoc adapter `CancellationToken` -> `AbortSignal`
 *
 * Note: then name of the class has to be exactly `AbortSignal` (this is some bad design):
 * https://github.com/node-fetch/node-fetch/issues/751#issue-582686301
 */
class AbortSignal implements IAbortSignal {
    subscriptions = new WeakMap<Listener, vscode.Disposable>();

    constructor(private readonly ct: vscode.CancellationToken) {}

    get aborted() {
        return this.ct.isCancellationRequested;
    }

    addEventListener(_type: "abort", listener: Listener, _opts?: unknown) {
        this.subscriptions.set(listener, this.ct.onCancellationRequested(listener, this));
    }

    removeEventListener(_type: "abort", listener: Listener) {
        this.subscriptions.get(listener)?.dispose();
    }

    // Some excess APIs that are not used by `node_fetch` impl
    onabort = null
    dispatchEvent() { return false; }
}
