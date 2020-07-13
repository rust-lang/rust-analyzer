import * as os from "os";
import * as vscode from 'vscode';
import * as path from 'path';
import * as ra from './lsp_ext';

import { Cargo } from './toolchain';
import { Ctx } from "./ctx";
import { currentRunnables, isDebuggable } from "./run";
import { RunnableEnvCfg } from "./config";

const debugOutput = vscode.window.createOutputChannel("Debug");
type DebugConfigProvider = (config: ProxyDebugConfiguration, executable: string, env: Record<string, string>, sourceFileMap?: Record<string, string>) => vscode.DebugConfiguration;

export async function makeDebugConfig(ctx: Ctx, runnable: ra.Runnable): Promise<void> {
    const scope = ctx.activeRustEditor?.document.uri;
    if (!scope) return;

    const debugConfig = proxyFromRunnable(runnable);
    if (!debugConfig) return;

    const wsLaunchSection = vscode.workspace.getConfiguration("launch", scope);
    const configurations = wsLaunchSection.get<any[]>("configurations") || [];

    const index = configurations.findIndex(c => c.name === debugConfig.name);
    if (index !== -1) {
        const answer = await vscode.window.showErrorMessage(`Launch configuration '${debugConfig.name}' already exists!`, 'Cancel', 'Update');
        if (answer === "Cancel") return;

        configurations[index] = debugConfig;
    } else {
        configurations.push(debugConfig);
    }

    await wsLaunchSection.update("configurations", configurations);
}

export async function startDebugSession(_ctx: Ctx, runnable: ra.Runnable): Promise<boolean> {
    let debugConfig: vscode.DebugConfiguration | undefined = undefined;
    let message = "";

    const wsLaunchSection = vscode.workspace.getConfiguration("launch");
    const configurations = wsLaunchSection.get<any[]>("configurations") || [];

    const index = configurations.findIndex(c => c.name === runnable.label);
    if (-1 !== index) {
        debugConfig = configurations[index];
        message = " (from launch.json)";
        debugOutput.clear();
    } else {
        try {
            debugConfig = proxyFromRunnable(runnable);
        } catch (err) {
            vscode.window.showErrorMessage(err);
        }
    }

    if (!debugConfig) return false;

    debugOutput.appendLine(`Launching debug configuration${message}:`);
    debugOutput.appendLine(JSON.stringify(debugConfig, null, 2));
    return vscode.debug.startDebugging(undefined, debugConfig);
}

function simplifyPath(p: string): string {
    const wsFolder = path.normalize(vscode.workspace.workspaceFolders![0].uri.fsPath); // folder exists or RA is not active.
    return path.normalize(p).replace(wsFolder, '${workspaceRoot}');
}

function expandPath(p: string): string {
    const wsFolder = path.normalize(vscode.workspace.workspaceFolders![0].uri.fsPath); // folder exists or RA is not active.
    return p.replace('${workspaceRoot}', wsFolder);
}


// async function getDebugConfiguration(ctx: Ctx, proxyCfg: ProxyDebugConfiguration): Promise<vscode.DebugConfiguration> {
//     const knownEngines: Record<string, DebugConfigProvider> = {
//         "vadimcn.vscode-lldb": getLldbDebugConfig,
//         "ms-vscode.cpptools": getCppvsDebugConfig
//     };
//     const debugOptions = ctx.config.debug;

//     let debugEngine = null;
//     if (debugOptions.engine === "auto") {
//         for (var engineId in knownEngines) {
//             debugEngine = vscode.extensions.getExtension(engineId);
//             if (debugEngine) break;
//         }
//     } else {
//         debugEngine = vscode.extensions.getExtension(debugOptions.engine);
//     }

//     if (!debugEngine) {
//         throw `Install [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)`
//             + ` or [MS C++ tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools) extension for debugging.`;
//     }

//     debugOutput.clear();
//     if (ctx.config.debug.openDebugPane) {
//         debugOutput.show(true);
//     }

//     const executable = await getDebugExecutable(runnable);
//     const env = prepareEnv(runnable, ctx.config.runnableEnv);
//     const debugConfig = knownEngines[debugEngine.id](runnable, simplifyPath(executable), env, debugOptions.sourceFileMap);
//     if (debugConfig.type in debugOptions.engineSettings) {
//         const settingsMap = (debugOptions.engineSettings as any)[debugConfig.type];
//         for (var key in settingsMap) {
//             debugConfig[key] = settingsMap[key];
//         }
//     }

//     if (debugConfig.name === "run binary") {
//         // The LSP side: crates\rust-analyzer\src\main_loop\handlers.rs,
//         // fn to_lsp_runnable(...) with RunnableKind::Bin
//         debugConfig.name = `run ${path.basename(executable)}`;
//     }

//     if (debugConfig.cwd) {
//         debugConfig.cwd = simplifyPath(debugConfig.cwd);
//     }

//     return debugConfig;
// }

// async function getDebugExecutable(runnable: ra.Runnable): Promise<string> {
//     const cargo = new Cargo(runnable.args.workspaceRoot || '.', debugOutput);
//     const executable = await cargo.executableFromArgs(runnable.args.cargoArgs);

//     // if we are here, there were no compilation errors.
//     return executable;
// }

function getLldbDebugConfig(proxyCfg: ProxyDebugConfiguration, executable: string, env: Record<string, string>, sourceFileMap?: Record<string, string>): vscode.DebugConfiguration {
    return {
        type: "lldb",
        request: "launch",
        name: proxyCfg.name,
        program: executable,
        args: proxyCfg.args,
        cwd: proxyCfg.cwd,
        sourceMap: sourceFileMap,
        sourceLanguages: ["rust"],
        env
    };
}

function getCppvsDebugConfig(proxyCfg: ProxyDebugConfiguration, executable: string, env: Record<string, string>, sourceFileMap?: Record<string, string>): vscode.DebugConfiguration {
    return {
        type: (os.platform() === "win32") ? "cppvsdbg" : "cppdbg",
        request: "launch",
        name: proxyCfg.label,
        program: executable,
        args: proxyCfg.args,
        cwd: proxyCfg.cwd,
        sourceFileMap,
        env,
    };
}

// These interfaces should be in sync with pacakge.json debuggers : rust : configurationAttributes
type CargoCommand = "run" | "test" | "bench";
interface CargoDebugConfiguration {
    command: CargoCommand,
    args?: string[],
    env?: Record<string, string>,
    cwd?: string,
}
interface ProxyDebugConfiguration extends vscode.DebugConfiguration {
    program?: string,
    cargo?: CargoDebugConfiguration,
    args?: string[],
    cwd?: string,
    env?: Record<string, string>,
    envFile?: string,
    debugEngineSettings?: Record<string, object>,
}

function proxyFromRunnable(runnable: ra.Runnable): ProxyDebugConfiguration | undefined {
    if (!isDebuggable(runnable)) return undefined;

    const proxyConfig: ProxyDebugConfiguration = {
        type: "rust",
        request: "launch",
        name: runnable.label,
        cargo: {
            command: runnable.args.cargoArgs[0] as CargoCommand,
            args: runnable.args.cargoArgs.slice(1),
        },
        args: runnable.args.executableArgs,
        cwd: runnable.args.workspaceRoot ? simplifyPath(runnable.args.workspaceRoot) : undefined,
    };

    return proxyConfig;
}

class ProxyConfigurationProvider implements vscode.DebugConfigurationProvider {
    constructor(readonly workspaceRoot: vscode.WorkspaceFolder, readonly context: Ctx) { }

    async provideDebugConfigurations?(_folder: vscode.WorkspaceFolder | undefined, _token?: vscode.CancellationToken): Promise<vscode.DebugConfiguration[]> {
        const defaultTargets: ProxyDebugConfiguration[] =
            [
                {
                    type: "rust",
                    request: "launch",
                    name: "Main binary",
                    cargo: {
                        command: "run"
                    }
                },
                {
                    type: "rust",
                    request: "launch",
                    name: "Tests",
                    cargo: {
                        command: "test"
                    }
                },
            ];

        const runnables = await currentRunnables(this.context);
        const targets = [...defaultTargets];
        runnables.forEach(it => {
            const proxyConfig = proxyFromRunnable(it);
            if (proxyConfig) {
                targets.push(proxyConfig);
            }
        });

        return targets;
    }

    async resolveDebugConfiguration?(folder: vscode.WorkspaceFolder | undefined, debugConfiguration: vscode.DebugConfiguration, _token?: vscode.CancellationToken): Promise<vscode.DebugConfiguration> {
        const proxyCfg = debugConfiguration as ProxyDebugConfiguration;
        const cwd = expandPath(proxyCfg.cwd || folder?.uri.fsPath || vscode.workspace.workspaceFolders![0].uri.fsPath);

        const knownEngines: Record<string, DebugConfigProvider> = {
            "vadimcn.vscode-lldb": getLldbDebugConfig,
            "ms-vscode.cpptools": getCppvsDebugConfig
        };
        const debugOptions = this.context.config.debug;

        let debugEngine = null;
        if (debugOptions.engine === "auto") {
            for (var engineId in knownEngines) {
                debugEngine = vscode.extensions.getExtension(engineId);
                if (debugEngine) break;
            }
        } else {
            debugEngine = vscode.extensions.getExtension(debugOptions.engine);
        }

        if (!debugEngine) {
            throw `Install [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)`
            + ` or [MS C++ tools](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools) extension for debugging.`;
        }

        debugOutput.clear();
        if (this.context.config.debug.openDebugPane) {
            debugOutput.show(true);
        }

        let executable: string;
        if (proxyCfg.cargo) {
            const cargoCwd = proxyCfg.cargo.cwd ? expandPath(proxyCfg.cargo.cwd) : cwd;
            const cargo = new Cargo(cargoCwd, debugOutput);
            const cargoArgs: string[] = [proxyCfg.cargo.command];
            if (proxyCfg.cargo.args) {
                cargoArgs.push(...proxyCfg.cargo.args);
            }
            executable = await cargo.executableFromArgs(cargoArgs);
        } else if (proxyCfg.program) {
            executable = proxyCfg.program;
        } else {
            throw `Invalid rust debug configuration: ${proxyCfg.name}`;
        }

        const env = prepareEnv(proxyCfg.name, proxyCfg.env, this.context.config.runnableEnv);
        const debugConfig = knownEngines[debugEngine.id](proxyCfg, simplifyPath(executable), env, debugOptions.sourceFileMap);
        if (debugConfig.type in debugOptions.engineSettings) {
            const settingsMap = (debugOptions.engineSettings as any)[debugConfig.type];
            for (var key in settingsMap) {
                debugConfig[key] = settingsMap[key];
            }
        }

        if (debugConfig.name === "run binary") {
            // The LSP side: crates\rust-analyzer\src\main_loop\handlers.rs,
            // fn to_lsp_runnable(...) with RunnableKind::Bin
            debugConfig.name = `run ${path.basename(executable)}`;
        }

        return debugConfig;
    }

    async resolveDebugConfigurationWithSubstitutedVariables?(_folder: vscode.WorkspaceFolder | undefined, debugConfiguration: vscode.DebugConfiguration, _token?: vscode.CancellationToken): Promise<vscode.DebugConfiguration> {
        return debugConfiguration;
    }
}

export function prepareEnv(name: string, explicitEnv: Record<string, string> | undefined, runnableEnvCfg: RunnableEnvCfg): Record<string, string> {
    const env = Object.assign({}, process.env as { [key: string]: string });

    if (runnableEnvCfg) {
        if (Array.isArray(runnableEnvCfg)) {
            for (const it of runnableEnvCfg) {
                if (!it.mask || new RegExp(it.mask).test(name)) {
                    Object.assign(env, it.env);
                }
            }
        } else {
            Object.assign(env, runnableEnvCfg);
        }
    }

    if (explicitEnv) Object.assign(env, explicitEnv);

    return env;
}


export function activateDebugConfigurationProvider(workspaceRoot: vscode.WorkspaceFolder, context: Ctx) {
    const provider = new ProxyConfigurationProvider(workspaceRoot, context);

    context.pushCleanup(vscode.debug.registerDebugConfigurationProvider("rust", provider, vscode.DebugConfigurationProviderTriggerKind.Dynamic));
}
