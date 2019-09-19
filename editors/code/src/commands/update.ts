import { exec, spawn } from 'child_process';
import { rename, unlink } from 'fs';
import { homedir } from 'os';
import { join } from 'path';
import { promisify } from 'util';
import { commands, window } from 'vscode';
import { Server } from '../server';

const renameAsync = promisify(rename);
const execAsync = promisify(exec);
const deleteAsync = promisify(unlink);

export async function updateServer(): Promise<void> {
    // Sanity check
    if (
        Server.config.raLspServerPath &&
        Server.config.raLspServerPath !== 'ra_lsp_server'
    ) {
        const result = await window.showWarningMessage(
            `Automatic updating cannot occur when ra_lsp_server is not running from '$HOME/.cargo/bin'. It is currently running from ${Server.config.raLspServerPath}. Would you like to continue?`,
            'Yes',
            'No (Recommended)'
        );
        if (result !== 'Yes') {
            return;
        }
    }
    window.showInformationMessage('Updating ra_lsp_server');
    try {
        const repo = Server.config.serverUpdateOptions.repo;
        const [
            { stdout: gitstdout },
            { stdout: cargostdout }
        ] = await Promise.all([
            execAsync(`git ls-remote ${repo} --ref master`),
            execAsync(`cargo install --list`)
        ]);
        if (isServerOutofDate(gitstdout, cargostdout)) {
            run(repo);
        } else {
            window.showInformationMessage(
                'ra_lsp_server is already up to date'
            );
        }
    } catch (error) {
        window.showErrorMessage(`Updating ra_lsp_server automatically failed:\n
${error}`);
    }
}

const newPath = join(__dirname, '../../prev_ra_executable');

function isServerOutofDate(gitstdout: string, cargostdout: string): boolean {
    const gitSha = gitstdout.slice(0, 8);
    const cargoSha = /^ra_lsp_server.+#\d{8}\):$/.exec(cargostdout);
    return cargoSha !== undefined || gitSha !== cargoSha;
}

async function run(repo: string) {
    try {
        // Handle windows not allowing
        await renameAsync(join(homedir(), '.cargo/bin/ra_lsp_server'), newPath);
    } catch (error) {
        // Fall through to next
    }
    window.showInformationMessage('Installing newer version');
    const spawned = spawn(
        `cargo install --git ${repo} ra_lsp_server --force ${Server.config.serverUpdateOptions.arguments}`
    );
    const channel = window.createOutputChannel(
        'Rust analyzer server update progress'
    );
    spawned.stdout.on('data', data => channel.append(data));
    await new Promise(resolve => spawned.on('exit', resolve));
    channel.dispose();
    window.showInformationMessage('Reloading server...');
    await commands.executeCommand('rust-analyzer.reload');
    try {
        await deleteAsync(newPath);
    } catch (error) {
        // Finish the promise
    }
}
