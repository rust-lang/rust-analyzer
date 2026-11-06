import * as vscode from "vscode";
import { findRustToolchainFile } from "./util";

/**
 * Matches the warning the server sends for each workspace whose toolchain is older than the
 * minimum it supports. Keep in sync with `GlobalState::check_workspaces_msrv`.
 */
const MSRV_WARNING_PATTERN =
    /^Workspace `(.+)` is using an outdated toolchain version `[^`]+` but rust-analyzer only supports/s;

/**
 * The amount of time in ms to wait after the last MSRV warning before showing them together.
 * The server sends one warning per workspace in quick succession after a workspace reload.
 */
const BATCH_DELAY_MS = 100;

/**
 * Returns the manifest path (or root, if there is no manifest) of the workspace that `message`
 * warns about, or `undefined` if `message` is not an MSRV warning.
 */
function parseMsrvWarning(message: string): string | undefined {
    return MSRV_WARNING_PATTERN.exec(message)?.[1];
}

/**
 * Creates the message for the MSRV warning notification from the list of affected workspaces
 */
function msrvWarningMessage(workspaces: readonly string[]): string {
    return (
        "Your workspace uses a rust-toolchain file with a toolchain too old for the extension shipped rust-analyzer to work properly. " +
        "Consider adding the rust-analyzer component to the toolchain file to use a compatible rust-analyzer version. " +
        "Add the following to your rust-toolchain file's `[toolchain]` section:\n" +
        'components = ["rust-analyzer"]\n' +
        `Workspaces: ${workspaces.join(", ")}`
    );
}

/**
 * Collects the MSRV warnings the server sends for each outdated workspace and shows the ones for
 * workspaces with a rust-toolchain file as a single notification.
 *
 * The notification text only depends on the set of affected workspaces, so VS Code replaces a
 * still visible notification instead of stacking a duplicate when the same set is reported again.
 */
export class MsrvWarningBatcher implements vscode.Disposable {
    /** Server messages keyed by the manifest path of the workspace they warn about. */
    private readonly pending = new Map<string, string>();
    private timer: NodeJS.Timeout | undefined;

    /**
     * Queues `message` to be shown with the other MSRV warnings in the batch.
     *
     * Returns `false` if `message` is not an MSRV warning.
     */
    add(message: string): boolean {
        const manifestPath = parseMsrvWarning(message);
        if (manifestPath === undefined) {
            return false;
        }
        this.pending.set(manifestPath, message);
        clearTimeout(this.timer);
        this.timer = setTimeout(() => void this.flush(), BATCH_DELAY_MS);
        return true;
    }

    dispose() {
        clearTimeout(this.timer);
    }

    private async flush() {
        this.timer = undefined;
        const warnings = [...this.pending];
        this.pending.clear();

        const workspaces = await Promise.all(
            warnings.map(async ([manifestPath, message]) => ({
                label: vscode.workspace.asRelativePath(manifestPath),
                message,
                toolchainFile: await findRustToolchainFile(manifestPath),
            })),
        );

        // Without a rust-toolchain file the advice does not apply, so show the server's message.
        for (const { message, toolchainFile } of workspaces) {
            if (!toolchainFile) {
                void vscode.window.showWarningMessage(message);
            }
        }

        const affected = workspaces
            .filter(({ toolchainFile }) => toolchainFile)
            .sort((left, right) => left.label.localeCompare(right.label));
        if (affected.length === 0) {
            return;
        }
        const toolchainFiles = new Map(
            affected.flatMap(({ toolchainFile }) =>
                toolchainFile ? [[toolchainFile.toString(), toolchainFile] as const] : [],
            ),
        );

        const openFile = "Open rust-toolchain file(s)";
        const choice = await vscode.window.showWarningMessage(
            msrvWarningMessage(affected.map(({ label }) => label)),
            openFile,
        );
        if (choice === openFile) {
            for (const toolchainFile of toolchainFiles.values()) {
                await vscode.window.showTextDocument(toolchainFile, { preview: false });
            }
        }
    }
}

export const _private = { parseMsrvWarning, msrvWarningMessage };
