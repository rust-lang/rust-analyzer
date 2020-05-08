import * as vscode from 'vscode';
import * as cp from 'child_process';
import { log } from "./util";

export type UpdatesChannel = "stable" | "nightly";

export const NIGHTLY_TAG = "nightly";

export class Config {
    readonly extensionId = "matklad.rust-analyzer";

    private readonly rootSection = "rust-analyzer";
    private readonly requiresReloadOpts = [
        "serverPath",
        "cargo",
        "procMacro",
        "files",
        "highlighting",
        "updates.channel",
        "rustupPath",
        "rustupChannel",
        "disableRustup"
    ]
        .map(opt => `${this.rootSection}.${opt}`);

    readonly package: {
        version: string;
        releaseTag: string | null;
        enableProposedApi: boolean | undefined;
    } = vscode.extensions.getExtension(this.extensionId)!.packageJSON;

    readonly globalStoragePath: string;

    constructor(ctx: vscode.ExtensionContext) {
        this.globalStoragePath = ctx.globalStoragePath;
        vscode.workspace.onDidChangeConfiguration(this.onDidChangeConfiguration, this, ctx.subscriptions);
        this.refreshLogging();
    }

    private refreshLogging() {
        log.setEnabled(this.traceExtension);
        log.debug(
            "Extension version:", this.package.version,
            "using configuration:", this.cfg
        );
    }

    private async onDidChangeConfiguration(event: vscode.ConfigurationChangeEvent) {
        this.refreshLogging();

        const requiresReloadOpt = this.requiresReloadOpts.find(
            opt => event.affectsConfiguration(opt)
        );

        if (!requiresReloadOpt) return;

        const userResponse = await vscode.window.showInformationMessage(
            `Changing "${requiresReloadOpt}" requires a reload`,
            "Reload now"
        );

        if (userResponse === "Reload now") {
            await vscode.commands.executeCommand("workbench.action.reloadWindow");
        }
    }

    // We don't do runtime config validation here for simplicity. More on stackoverflow:
    // https://stackoverflow.com/questions/60135780/what-is-the-best-way-to-type-check-the-configuration-for-vscode-extension

    private get cfg(): vscode.WorkspaceConfiguration {
        return vscode.workspace.getConfiguration(this.rootSection);
    }

    /**
     * Beware that postfix `!` operator erases both `null` and `undefined`.
     * This is why the following doesn't work as expected:
     *
     * ```ts
     * const nullableNum = vscode
     *  .workspace
     *  .getConfiguration
     *  .getConfiguration("rust-analyer")
     *  .get<number | null>(path)!;
     *
     * // What happens is that type of `nullableNum` is `number` but not `null | number`:
     * const fullFledgedNum: number = nullableNum;
     * ```
     * So this getter handles this quirk by not requiring the caller to use postfix `!`
     */
    private get<T>(path: string): T {
        return this.cfg.get<T>(path)!;
    }

    get serverPath() { return this.get<null | string>("serverPath"); }
    get channel() { return this.get<UpdatesChannel>("updates.channel"); }
    get askBeforeDownload() { return this.get<boolean>("updates.askBeforeDownload"); }
    get traceExtension() { return this.get<boolean>("trace.extension"); }

    get inlayHints() {
        return {
            enable: this.get<boolean>("inlayHints.enable"),
            typeHints: this.get<boolean>("inlayHints.typeHints"),
            parameterHints: this.get<boolean>("inlayHints.parameterHints"),
            chainingHints: this.get<boolean>("inlayHints.chainingHints"),
            maxLength: this.get<null | number>("inlayHints.maxLength"),
        };
    }

    get checkOnSave() {
        return {
            command: this.get<string>("checkOnSave.command"),
        };
    }

    get debug() {
        // "/rustc/<id>" used by suggestions only.
        const { ["/rustc/<id>"]: _, ...sourceFileMap } = this.get<Record<string, string>>("debug.sourceFileMap");

        return {
            engine: this.get<string>("debug.engine"),
            engineSettings: this.get<object>("debug.engineSettings"),
            openUpDebugPane: this.get<boolean>("debug.openUpDebugPane"),
            sourceFileMap: sourceFileMap,
        };
    }

    get rustupDisabled(): boolean {
        return this.get<boolean>('disableRustup');
    }

    get rustupPath(): string {
        return this.get<string>('rustupPath');
    }

    private parseActiveToolchain(rustupOutput: string): string {
        // There may a default entry under 'installed toolchains' section, so search
        // for currently active/overridden one only under 'active toolchain' section
        const activeToolchainsIndex = rustupOutput.search('active toolchain');
        if (activeToolchainsIndex !== -1) {
            rustupOutput = rustupOutput.substr(activeToolchainsIndex);

            const matchActiveChannel = /^(\S*) \((?:default|overridden)/gm;
            const match = matchActiveChannel.exec(rustupOutput);
            if (!match) {
                throw new Error(
                    `couldn't find active toolchain under 'active toolchains'`,
                );
            } else if (matchActiveChannel.exec(rustupOutput)) {
                throw new Error(
                    `multiple active toolchains found under 'active toolchains'`,
                );
            }

            return match[1];
        }

        // Try matching the third line as the active toolchain
        const match = /^(?:.*\r?\n){2}(\S*) \((?:default|overridden)/.exec(
            rustupOutput,
        );
        if (match) {
            return match[1];
        }

        throw new Error(`couldn't find active toolchains`);
    }

    private getActiveChannel(wsPath: string): string {
        // rustup info might differ depending on where it's executed
        // (e.g. when a toolchain is locally overriden), so executing it
        // under our current workspace root should give us close enough result

        let activeChannel;
        try {
            // `rustup show active-toolchain` is available since rustup 1.12.0
            activeChannel = cp
                .execSync(`${this.rustupPath} show active-toolchain`, {
                    cwd: wsPath,
                })
                .toString()
                .trim();
            // Since rustup 1.17.0 if the active toolchain is the default, we're told
            // by means of a " (default)" suffix, so strip that off if it's present
            // If on the other hand there's an override active, we'll get an
            // " (overridden by ...)" message instead.
            activeChannel = activeChannel.replace(/ \(.*\)$/, '');
        } catch (e) {
            // Possibly an old rustup version, so try rustup show
            const showOutput = cp.execSync(`${this.rustupPath} show`, {
                cwd: wsPath,
            })
                .toString();
            activeChannel = this.parseActiveToolchain(showOutput);
        }

        return activeChannel;
    }

    public getRustupChannel(wsPath: string): string {
        const channel = this.get<string>('rustupChannel');
        if (channel === 'default' || !channel) {
            try {
                return this.getActiveChannel(wsPath);
            } catch (e) {
                // rustup might not be installed at the time the configuration is
                // initially loaded, so silently ignore the error and return a default value
                return 'nightly';
            }
        } else {
            return channel;
        }
    }
}
