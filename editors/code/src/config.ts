import * as vscode from 'vscode';
import { Env } from './client';
import { log } from "./util";

export type UpdatesChannel = "stable" | "nightly";

const NIGHTLY_TAG = "nightly";

export type RunnableEnvCfg = undefined | Record<string, string> | { mask?: string; env: Record<string, string> }[];

export class Config {
    readonly extensionId = "matklad.rust-analyzer";

    readonly rootSection = "rust-analyzer";
    private readonly requiresReloadOpts = [
        "serverPath",
        "server",
        "cargo",
        "procMacro",
        "files",
        "lens", // works as lens.*
    ]
        .map(opt => `${this.rootSection}.${opt}`);

    readonly package: {
        version: string;
        releaseTag: string | null;
        enableProposedApi: boolean | undefined;
    } = vscode.extensions.getExtension(this.extensionId)!.packageJSON;

    readonly globalStorageUri: vscode.Uri;

    constructor(ctx: vscode.ExtensionContext) {
        this.globalStorageUri = ctx.globalStorageUri;
        vscode.workspace.onDidChangeConfiguration(this.onDidChangeConfiguration, this, ctx.subscriptions);
        this.refreshLogging();
    }

    private refreshLogging() {
        log.setEnabled(this.traceExtension);
        log.info("Extension version:", this.package.version);

        const cfg = Object.entries(this.cfg).filter(([_, val]) => !(val instanceof Function));
        log.info("Using configuration", Object.fromEntries(cfg));
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
     *  .getConfiguration("rust-analyzer")
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

    get serverPath() {
        return this.get<null | string>("server.path") ?? this.get<null | string>("serverPath");
    }
    get serverExtraEnv() { return this.get<Env | null>("server.extraEnv") ?? {}; }
    get traceExtension() { return this.get<boolean>("trace.extension"); }

    get cargoRunner() {
        return this.get<string | undefined>("cargoRunner");
    }

    get runnableEnv() {
        return this.get<RunnableEnvCfg>("runnableEnv");
    }

    get debug() {
        let sourceFileMap = this.get<Record<string, string> | "auto">("debug.sourceFileMap");
        if (sourceFileMap !== "auto") {
            // "/rustc/<id>" used by suggestions only.
            const { ["/rustc/<id>"]: _, ...trimmed } = this.get<Record<string, string>>("debug.sourceFileMap");
            sourceFileMap = trimmed;
        }

        return {
            engine: this.get<string>("debug.engine"),
            engineSettings: this.get<object>("debug.engineSettings"),
            openDebugPane: this.get<boolean>("debug.openDebugPane"),
            sourceFileMap: sourceFileMap
        };
    }

    get hoverActions() {
        return {
            enable: this.get<boolean>("hoverActions.enable"),
            implementations: this.get<boolean>("hoverActions.implementations.enable"),
            references: this.get<boolean>("hoverActions.references.enable"),
            run: this.get<boolean>("hoverActions.run.enable"),
            debug: this.get<boolean>("hoverActions.debug.enable"),
            gotoTypeDef: this.get<boolean>("hoverActions.gotoTypeDef.enable"),
        };
    }

    get currentExtensionIsNightly() {
        return this.package.releaseTag === NIGHTLY_TAG;
    }
}

export async function updateConfig(config: vscode.WorkspaceConfiguration) {
    const renames = [
        ["assist.allowMergingIntoGlobImports", "imports.merge.glob",],
        ["assist.exprFillDefault", "assist.expressionFillDefault",],
        ["assist.importEnforceGranularity", "imports.granularity.enforce",],
        ["assist.importGranularity", "imports.granularity.group",],
        ["assist.importMergeBehavior", "imports.granularity.group",],
        ["assist.importMergeBehaviour", "imports.granularity.group",],
        ["assist.importGroup", "imports.group.enabled",],
        ["assist.importPrefix", "imports.prefix",],
        ["cache.warmup", "primeCaches.enabled",],
        ["cargo.loadOutDirsFromCheck", "cargo.buildScripts.enabled",],
        ["cargo.runBuildScripts", "cargo.buildScripts.enabled",],
        ["cargo.runBuildScriptsCommand", "cargo.buildScripts.overrideCommand",],
        ["cargo.useRustcWrapperForBuildScripts", "cargo.buildScripts.useRustcWrapper",],
        ["completion.snippets", "completion.snippets.custom",],
        ["diagnostics.enableExperimental", "diagnostics.experimental.enabled",],
        ["experimental.procAttrMacros", "procMacro.attributes.enabled",],
        ["highlighting.strings", "semanticHighlighting.strings.enabled",],
        ["highlightRelated.breakPoints", "highlightRelated.breakPoints.enabled",],
        ["highlightRelated.exitPoints", "highlightRelated.exitPoints.enabled",],
        ["highlightRelated.yieldPoints", "highlightRelated.yieldPoints.enabled",],
        ["highlightRelated.references", "highlightRelated.references.enabled",],
        ["hover.documentation", "hover.documentation.enabled",],
        ["hover.linksInHover", "hover.links.enabled",],
        ["hoverActions.linksInHover", "hover.links.enabled",],
        ["hoverActions.debug", "hoverActions.debug.enabled",],
        ["hoverActions.enable", "hoverActions.enabled",],
        ["hoverActions.gotoTypeDef", "hoverActions.gotoTypeDef.enabled",],
        ["hoverActions.implementations", "hoverActions.implementations.enabled",],
        ["hoverActions.references", "hoverActions.references.enabled",],
        ["hoverActions.run", "hoverActions.run.enabled",],
        ["inlayHints.chainingHints", "inlayHints.chainingHints.enabled",],
        ["inlayHints.closureReturnTypeHints", "inlayHints.closureReturnTypeHints.enabled",],
        ["inlayHints.hideNamedConstructorHints", "inlayHints.typeHints.hideNamedConstructorHints",],
        ["inlayHints.parameterHints", "inlayHints.parameterHints.enabled",],
        ["inlayHints.reborrowHints", "inlayHints.reborrowHints.enabled",],
        ["inlayHints.typeHints", "inlayHints.typeHints.enabled",],
        ["lruCapacity", "lru.capacity",],
        ["runnables.cargoExtraArgs", "runnables.extraArgs",],
        ["runnables.overrideCargo", "runnables.command",],
        ["rustcSource", "rustc.source",],
        ["rustfmt.enableRangeFormatting", "rustfmt.rangeFormatting.enabled"]
    ];

    for (const [oldKey, newKey] of renames) {
        const inspect = config.inspect(oldKey);
        if (inspect !== undefined) {
            const valMatrix = [
                { val: inspect.globalValue, langVal: inspect.globalLanguageValue, target: vscode.ConfigurationTarget.Global },
                { val: inspect.workspaceFolderValue, langVal: inspect.workspaceFolderLanguageValue, target: vscode.ConfigurationTarget.WorkspaceFolder },
                { val: inspect.workspaceValue, langVal: inspect.workspaceLanguageValue, target: vscode.ConfigurationTarget.Workspace }
            ];
            for (const { val, langVal, target } of valMatrix) {
                const pred = (val: unknown) => {
                    // some of the updates we do only append "enable" or "custom"
                    // that means on the next run we would find these again, but as objects with
                    // these properties causing us to destroy the config
                    // so filter those already updated ones out
                    return val !== undefined && !(typeof val === "object" && val !== null && (val.hasOwnProperty("enable") || val.hasOwnProperty("custom")));
                };
                if (pred(val)) {
                    await config.update(newKey, val, target, false);
                    await config.update(oldKey, undefined, target, false);
                }
                if (pred(langVal)) {
                    await config.update(newKey, langVal, target, true);
                    await config.update(oldKey, undefined, target, true);
                }
            }
        }
    }
}
