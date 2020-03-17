import * as vscode from "vscode";
import * as path from "path";
import * as fs from 'fs';

import { vscodeReinstallExtension, vscodeReloadWindow, log, vscodeInstallExtensionFromVsix, assert, waitForCancellation } from "../util";
import { Config, UpdatesChannel } from "../config";
import { ArtifactReleaseInfo, ArtifactSource } from "./interfaces";
import { downloadArtifactWithProgressUi } from "./downloads";
import { fetchArtifactReleaseInfo } from "./fetch_artifact_release_info";
import { PersistentState } from "../persistent_state";

const HEURISTIC_NIGHTLY_RELEASE_PERIOD_IN_HOURS = 25;

/**
 * Installs `stable` or latest `nightly` version or does nothing if the current
 * extension version is what's needed according to `desiredUpdateChannel`.
 */
export async function ensureProperExtensionVersion(
    config: Config,
    state: PersistentState,
    ct: vscode.CancellationToken
): Promise<never | void> {

    // User has built lsp server from sources, she should manage updates manually
    if (config.serverSource?.type === ArtifactSource.Type.ExplicitPath) return;

    const currentUpdChannel = config.installedExtensionUpdateChannel;
    const desiredUpdChannel = config.updatesChannel;

    if (currentUpdChannel === UpdatesChannel.Stable) {
        // Release date is present only when we are on nightly
        await state.installedNightlyExtensionReleaseDate.set(null);
    }

    if (ct.isCancellationRequested) return;

    if (desiredUpdChannel === UpdatesChannel.Stable) {
        // VSCode should handle updates for stable channel
        if (currentUpdChannel === UpdatesChannel.Stable) return;

        if (!await askToDownloadProperExtensionVersion(config, ct)) return;

        await vscodeReinstallExtension(config.extensionId);
        await vscodeReloadWindow(); // never returns
    }

    if (currentUpdChannel === UpdatesChannel.Stable) {
        if (!await askToDownloadProperExtensionVersion(config, ct)) {
            return;
        }

        return await tryDownloadNightlyExtension(config, state, ct, () => true);
    }

    const currentExtReleaseDate = state.installedNightlyExtensionReleaseDate.get();

    if (currentExtReleaseDate === null) {
        void vscode.window.showErrorMessage(
            "Nightly release date must've been set during the installation. " +
            "Did you download and install the nightly .vsix package manually?"
        );
        throw new Error("Nightly release date was not set in globalStorage");
    }

    const dateNow = new Date;
    const hoursSinceLastUpdate = diffInHours(currentExtReleaseDate, dateNow);
    log.debug(
        "Current rust-analyzer nightly was downloaded", hoursSinceLastUpdate,
        "hours ago, namely:", currentExtReleaseDate, "and now is", dateNow
    );

    if (hoursSinceLastUpdate < HEURISTIC_NIGHTLY_RELEASE_PERIOD_IN_HOURS) {
        return;
    }
    if (!await askToDownloadProperExtensionVersion(config, ct, "The installed nightly version is most likely outdated. ")) {
        return;
    }

    await tryDownloadNightlyExtension(config, state, ct, releaseInfo => {
        assert(
            currentExtReleaseDate.getTime() === state.installedNightlyExtensionReleaseDate.get()?.getTime(),
            "Other active VSCode instance has reinstalled the extension"
        );

        if (releaseInfo.releaseDate.getTime() === currentExtReleaseDate.getTime()) {
            vscode.window.showInformationMessage(
                "Whoops, it appears that your nightly version is up-to-date. " +
                "There might be some problems with the upcomming nightly release " +
                "or you traveled too far into the future. Sorry for that 😅! "
            );
            return false;
        }
        return true;
    });
}

async function askToDownloadProperExtensionVersion(
    config: Config,
    ct: vscode.CancellationToken,
    reason = "",
) {
    if (!config.askBeforeDownload) return true;

    const stableOrNightly = config.updatesChannel === UpdatesChannel.Stable ? "stable" : "latest nightly";

    // When the cancellation is requested the information message is not dismissed.
    // Unfortunately there is no API for dismissing it:
    // https://github.com/Microsoft/vscode/issues/2732
    // https://github.com/microsoft/vscode/issues/50232
    // It will just hang in user's notification bar and be ignored even if intercated with

    return await Promise.race([
        vscode.window.showInformationMessage(
            reason + `Do you want to download the ${stableOrNightly} rust-analyzer extension ` +
            `version and reload the window now?`,
            "Download now", "Cancel"
        ).then(userResponse => userResponse === "Download now"),

        waitForCancellation(ct).then(() => false)
    ]);
}

/**
 * Shutdowns the process in case of success (i.e. reloads the window) or throws an error.
 */
async function tryDownloadNightlyExtension(
    config: Config,
    state: PersistentState,
    ct: vscode.CancellationToken,
    shouldDownload: (releaseInfo: ArtifactReleaseInfo) => boolean
): Promise<never | void> {
    const vsixSource = config.nightlyVsixSource;
    try {
        const releaseInfo = await fetchArtifactReleaseInfo(vsixSource.repo, vsixSource.file, vsixSource.tag);

        if (ct.isCancellationRequested || !shouldDownload(releaseInfo)) return;

        if (!await downloadArtifactWithProgressUi(releaseInfo, vsixSource.file, vsixSource.dir, "nightly extension", ct)) {
            return;
        }

        const vsixPath = path.join(vsixSource.dir, vsixSource.file);

        // The following 4 lines of code shoud be atomic and syncronous
        // But we cannot syncronously await promises...

        await vscodeInstallExtensionFromVsix(vsixPath);
        await state.installedNightlyExtensionReleaseDate.set(releaseInfo.releaseDate);
        fs.unlinkSync(vsixPath);

        await vscodeReloadWindow(); // never returns
    } catch (err) {
        log.downloadError(err, "nightly extension", vsixSource.repo.name);
    }
};

function diffInHours(a: Date, b: Date): number {
    // Discard the time and time-zone information (to abstract from daylight saving time bugs)
    // https://stackoverflow.com/a/15289883/9259330

    const utcA = Date.UTC(a.getFullYear(), a.getMonth(), a.getDate());
    const utcB = Date.UTC(b.getFullYear(), b.getMonth(), b.getDate());

    return (utcA - utcB) / (1000 * 60 * 60);
}
