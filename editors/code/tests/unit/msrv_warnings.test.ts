import * as assert from "assert";
import { _private } from "../../src/msrv_warnings";
import type { Context } from ".";

/** Mirrors the message built by `GlobalState::check_workspaces_msrv`. */
function serverMessage(manifestPath: string): string {
    return (
        `Workspace \`${manifestPath}\` is using an outdated toolchain version \`1.80.0\` but rust-analyzer only supports \`1.94.0\` and higher.\n` +
        "Consider using the rust-analyzer rustup component for your toolchain or\n" +
        "                        upgrade your toolchain to a supported version.\n\n"
    );
}

const ADVICE =
    "Your workspace uses a rust-toolchain file with a toolchain too old for the extension shipped rust-analyzer to work properly. " +
    "Consider adding the rust-analyzer component to the toolchain file to use a compatible rust-analyzer version. " +
    "Add the following to your rust-toolchain file's `[toolchain]` section:\n" +
    'components = ["rust-analyzer"]';

export async function getTests(ctx: Context) {
    await ctx.suite("MSRV warnings/Parse server message", (suite) => {
        suite.addTest("Cargo workspace", async () => {
            assert.deepStrictEqual(
                _private.parseMsrvWarning(serverMessage("/home/user/repo/Cargo.toml")),
                "/home/user/repo/Cargo.toml",
            );
        });

        suite.addTest("Path containing a backtick", async () => {
            assert.deepStrictEqual(
                _private.parseMsrvWarning(serverMessage("/home/user/re`po/Cargo.toml")),
                "/home/user/re`po/Cargo.toml",
            );
        });

        suite.addTest("Other server message", async () => {
            assert.deepStrictEqual(
                _private.parseMsrvWarning("Failed to discover workspace."),
                undefined,
            );
        });
    });

    await ctx.suite("MSRV warnings/Notification text", (suite) => {
        suite.addTest("Single workspace", async () => {
            assert.deepStrictEqual(
                _private.msrvWarningMessage(["repo/Cargo.toml"]),
                `${ADVICE}\nWorkspaces: repo/Cargo.toml`,
            );
        });

        suite.addTest("Multiple workspaces", async () => {
            assert.deepStrictEqual(
                _private.msrvWarningMessage(["repo-a/Cargo.toml", "repo-b/Cargo.toml"]),
                `${ADVICE}\nWorkspaces: repo-a/Cargo.toml, repo-b/Cargo.toml`,
            );
        });
    });
}
