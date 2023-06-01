import * as vscode from "vscode";
import { testController } from ".";
import type * as ra from "../lsp_ext";
import { assert, assertNever, isCargoTomlDocument, isRustDocument, sleep } from "../util";
import { RaApiHelper } from "./api_helper";
import { RunnableFacde } from "./RunnableFacde";
import type { CargoMetadata } from "../toolchain";
import {
    type CargoPackageNode,
    type CargoWorkspaceNode,
    TargetNode,
    NodeKind,
    TestModuleNode,
    isTestModuleNode,
    WorkspacesWalker,
    TestNode,
    type Nodes,
    TargetKind,
    type TestLikeNode,
    isTestNode,
    isTestLikeNode,
    DummyRootNode,
    UriMatcher,
} from "./test_model_tree";
import { fail } from "assert";

export const disposiables: vscode.Disposable[] = [];

async function discoverAllFilesInWorkspaces() {
    if (!vscode.workspace.workspaceFolders) {
        return;
    }

    await refreshCore();
}

function registerWatcherForWorkspaces() {
    if (!vscode.workspace.workspaceFolders) {
        return;
    }

    // listen to document changes to re-parse unsaved changes:
    const disposable = vscode.workspace.onDidChangeTextDocument(async e => {
        const document = e.document;

        if (isRustDocument(document)) {
            await handleRustFileChange(document.uri);
            return;
        }

        if (isCargoTomlDocument(document)) {
            await handleRustProjectFileEvent(e.document.uri);
            return;
        }
    });
    disposiables.push(disposable);

    vscode.workspace.workspaceFolders
        .map(watchWorkspace);
}

function registerActiveTextEditor() {
    const disposable = vscode.window.onDidChangeActiveTextEditor(onDidChangeActiveTextEditorForTestExplorer);
    disposiables.push(disposable);
}

/**
 * whether the file is already loaded in test model tree
 */
function isRustFileAlreadyLoaded(uri: vscode.Uri) {
    const nodes = UriMatcher.match(uri, DummyRootNode.instance);
    return !!(nodes[0] && nodes[0].testChildren.size > 0);
}

async function onDidChangeActiveTextEditorForTestExplorer(e: vscode.TextEditor | undefined) {
    if (!testController) return;

    if (!e) {
        return;
    }

    if (isRustDocument(e.document)) {
        const isDocumentLoaded = isRustFileAlreadyLoaded(e.document.uri);
        if (isDocumentLoaded) {
            return;
        }

        // if the file is still not loaded yet
        // as if the file is changed, to update its related info, immediately.
        await handleRustFileChangeCore(e.document.uri);
        return;
    }
};

// Not watch the change of file(the disk), instead, use `onDidChangeTextDocument` to watch the editor(the memory of VSCode)
//
// This also means, please do not use other ways to change the file, such as in terminal or another editor
// However, VSCode would trigger `onDidChangeTextDocument` for an opened file when you change and saved it in other place
//
// Because:
// 1. if auto-save is enabled, the event would be triggered twice, then we need to give a longer debounce time(more than 1s) to avoid duplicate work
// 2. For now, RA is synced with VSCode rather than disk
//     2.2 A change in disk is confused in fact. Let's say you have content A on dist, content B on VSCode. You save content C on disk now.
//         What should we do? What should VSCode do? Should VSCode send content C to RA thourgh LSP? If so, it would be inconsistant with the content in VSCode!
function watchWorkspace(workspaceFolder: vscode.WorkspaceFolder) {
    const rsRrojectWatcher = watchRustProjectFileChange(workspaceFolder);
    const rsFileWatcher = watchRustFileChange(workspaceFolder);
    disposiables.push(rsRrojectWatcher);
    disposiables.push(rsFileWatcher);

    // For now, the only supported project file is cargo.
    function watchRustProjectFileChange(workspaceFolder: vscode.WorkspaceFolder): vscode.FileSystemWatcher {
        const pattern = new vscode.RelativePattern(workspaceFolder, '**/Cargo.toml');
        const watcher = vscode.workspace.createFileSystemWatcher(
            pattern,
            false,
            true, // not listen to change event in fact
            false
        );
        watcher.onDidCreate(handleRustProjectFileEvent);
        watcher.onDidDelete(handleRustProjectFileEvent);
        return watcher;
    }

    function watchRustFileChange(workspaceFolder: vscode.WorkspaceFolder) {
        const pattern = new vscode.RelativePattern(workspaceFolder, '**/*.rs');
        const watcher = vscode.workspace.createFileSystemWatcher(
            pattern,
            false,
            true,  // not listen to change event in fact
            false
        );
        watcher.onDidCreate(handleRustFileCreate);
        watcher.onDidDelete(handleRustFileDelete);
        return watcher;
    }
}

// refresh all things if the project file is added/changed/deleted
// Because we do not know whther the change would
//     - change packages
//     - change targets(.e.g, changing bin file path)
function handleRustProjectFileEvent(uri: vscode.Uri) {
    // We need to order this after language server updates, but there's no API for that.
    // debounce will wait a short time
    debounceRefreshCore();
}

async function handleRustFileCreate(uri: vscode.Uri) {
    // We need to order this after language server updates, but there's no API for that.
    // Hence, good old sleep().
    await sleep(20);
    await loadFileAndUpdateModel(uri);
    updateTestItemsByModel();
}

async function handleRustFileChange(uri: vscode.Uri) {
    // We need to order this after language server updates, but there's no API for that.
    // debounce will wait a short time
    debounceHandleRustFileChangeCore(uri);
}

async function handleRustFileDelete(uri: vscode.Uri) {
    // We need to order this after language server updates, but there's no API for that.
    // Hence, good old sleep().
    await sleep(20);
    DummyRootNode.instance.removeTestItemsRecursivelyByUri(uri);
    updateTestItemsByModel();
}

const FILE_DEBOUNCE_DELAY_MS = 500; // 0.5s, assume charactor typing speed is 2/s

// FIXME: if there are changes in two files, we will lost the first change. But it would rarely happen
function debounce(fn: Function, ms: number) {
    let timeout: NodeJS.Timeout | undefined = undefined;
    return (...params: any[]) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => {
            fn(...params);
        }, ms);
    };
}

// FIXME: if there are changes in two files, we will lost the first chagne
const debounceHandleRustFileChangeCore = debounce(handleRustFileChangeCore, FILE_DEBOUNCE_DELAY_MS);
const debounceRefreshCore = debounce(refreshCore, FILE_DEBOUNCE_DELAY_MS);

export async function refreshHandler() {
    await refreshCore();
}

async function refreshCore() {
    if (!testController) return;

    // Discard all
    // should we discard the old model tree here? should the user see the previous test items when refreshing?
    // testController.items.replace([]);
    DummyRootNode.instance.clear();

    const cargoMetadataArray = await RaApiHelper.cargoWorkspaces();

    if (!cargoMetadataArray) return;

    // The workspaces got from RA contains depdency packages(.i.e, RA does not add "--no-deps" when running `cargo metadata`)
    // But they are not needed in test explorer
    const noDepsWorkspaces = cargoMetadataArray.map(filterOutDepdencyPackages);

    DummyRootNode.instance.initByMedatada(noDepsWorkspaces);

    // After init, the target might not conatins any test(rather than not-fetched tests)
    // So we could not collect nodes which children need to be fetched, and fetch them
    // Instead, we pretend the behavior they are changed, so that the empty target will be removed

    const allTargetUris = noDepsWorkspaces.flatMap(it =>
        it.packages
            .flatMap(p => p.targets)
            .map(target => target.src_path)
            .map(vscode.Uri.file)
    );

    for (const uri of allTargetUris) {
        await loadFileAndUpdateModel(uri);
    }

    // update all test info in current file, and trigger build of test item tree
    await onDidChangeActiveTextEditorForTestExplorer(vscode.window.activeTextEditor);

    function filterOutDepdencyPackages(metadata: CargoMetadata) {
        return {
            ...metadata,
            packages: metadata.packages.filter(p =>
                metadata.workspace_members.includes(p.id)
            )
        };
    }
}

async function handleRustFileChangeCore(uri: vscode.Uri) {
    await loadFileAndUpdateModel(uri);
    updateTestItemsByModel();
}

export const resolveHandler  = async function (item:vscode.TestItem|undefined) {
    if (!item) {
        // init logic
        await discoverAllFilesInWorkspaces();

        registerWatcherForWorkspaces();
        registerActiveTextEditor();
        return;
    }

    assert(!!item.uri, "Always give a uri to a test item");


    const node = getTestModelByTestItem(item);

    switch (node.kind) {
        case NodeKind.DummyRoot:
            fail("Dummy root should never be visited");
        case NodeKind.CargoWorkspace:
            fail("Package data is got when getting workspace data, no need to be resolved lazily");
        case NodeKind.CargoPackage:
            fail("Targets data is got when getting workspace data, no need to be resolved lazily");
        case NodeKind.Target:
            fail("The children for target are handled specially. Target is the surface of cargo metadata and front-end life-cycle for now. Eagerly fetch the children to verify whether there is tests or not.");
        case NodeKind.TestModule:
            if (node.testChildren.size > 0) {
                return;
            }
            item.busy = true;
            await fetchAndUpdateChildrenForTestModuleNode(node);
            item.busy = false;
            break;
        case NodeKind.Test:
            fail("test does not contain any children, and should not be be able to resolve.");
    }

    // WorkspacesPrinter.print(node);

    // add the new test items to existing test item tree
    VscodeTestTreeBuilder.buildChildrenFor(node);
};

// Rebuild the whole test item tree
function updateTestItemsByModel() {
    assert(!!testController);
    testController.items.replace([]);
    // WorkspacesPrinter.print(DummyRootNode.instance);
    const rootTestItems = VscodeTestTreeBuilder.build();
    testController.items.replace(rootTestItems);
}

async function getNormalizedTestRunnablesInFile(uri: vscode.Uri) {
    const rawRunables = await RaApiHelper.getTestRunnablesInFile(uri);

    assert(!!rawRunables);

    const runnables = rawRunables.map(it => new RunnableFacde(it));

    // User might copy and paste test, and then there might be same name test or test module
    // Although it's wrong, we need to tolerate it.
    // pick the first one.
    return uniqueRunnables(runnables);

    function uniqueRunnables(runnables: RunnableFacde[]) {
        const map = new Map<string, RunnableFacde>();
        runnables.forEach(runnable => {
            const key = `${runnable.workspaceRoot}|${runnable.packageName}|${runnable.targetKind}|${runnable.targetName}|${runnable.origin.label}`;
            if (!map.has(key)) {
                map.set(key, runnable);
            }
        });
        return Array.from(map.values());
    }
}

async function loadFileAndUpdateModel(uri: vscode.Uri) {
    const runnables = await getNormalizedTestRunnablesInFile(uri);

    // Maybe from some to none
    // need to recursively clean the parent, until there is at least one test cases.
    if (runnables.length === 0) {
        DummyRootNode.instance.removeTestItemsRecursivelyByUri(uri);
        return;
    }

    const testModuelRunnables = runnables.filter(it =>
        it.testKind === NodeKind.TestModule)
        .sort(RunnableFacde.sortByLabel);

    const testItemRunnables = runnables.filter(it =>
        it.testKind === NodeKind.Test);

    assert(testModuelRunnables.length + testItemRunnables.length === runnables.length);

    // FIXME: should be file test modules, because of `path` attribute
    const fileTestModuleRunnbale = testModuelRunnables[0]!;

    const nearestNode = DummyRootNode.instance.findNearestNodeByRunnable(fileTestModuleRunnbale);

    assert(nearestNode.kind !== NodeKind.Test, "it's a test module");
    assert(nearestNode.kind !== NodeKind.CargoWorkspace, "We never delete workspace and package info unless refresh, so at least it's a package");

    // create target node when creating the first test for some target.
    // This is necessary, because we do not know how many targets a package contains unless we fetch data thorugh `cargo metadata`
    // But we want to only fetch it when cargo file is changed, to make things more lazily.
    if (fileTestModuleRunnbale.origin.label === "test-mod "
        && nearestNode.kind !== NodeKind.TestModule) {
        assert(nearestNode.kind === NodeKind.CargoPackage, "we do not delete package node unless refetch metadata");
        // This runnable is from a target, create the target if it's not exist in test model tree
        const newTargetNode = new TargetNode(nearestNode,
            fileTestModuleRunnbale.targetKind,
            fileTestModuleRunnbale.targetName,
            fileTestModuleRunnbale.uri.fsPath);
        nearestNode.targets.add(newTargetNode);
    }

    await ensureTestModuleParentExist(fileTestModuleRunnbale);

    const parentModule = DummyRootNode.instance.findNearestNodeByRunnable(fileTestModuleRunnbale);

    assert(isTestModuleNode(parentModule));

    await updateFileDefinitionTestModuleByRunnables(parentModule, runnables);

    async function ensureTestModuleParentExist(runnable: RunnableFacde) {
        let nearestNode = DummyRootNode.instance.findNearestNodeByRunnable(fileTestModuleRunnbale);

        assert(isTestLikeNode(nearestNode));

        while (!isTestNodeAndRunnableMatched(nearestNode, runnable)) {
            // parent test node is not existed, create it recursively
            assert(isTestModuleNode(nearestNode));
            await fetchAndUpdateChildrenForTestModuleNode(nearestNode);

            nearestNode = DummyRootNode.instance.findNearestNodeByRunnable(fileTestModuleRunnbale);
            assert(isTestLikeNode(nearestNode));
        }
    }
}

async function fetchAndUpdateChildrenForTestModuleNode(testModuleNode: TestModuleNode) {
    assert(
        testModuleNode.isDummyTestModule() ===
        (testModuleNode.declarationInfo.uri.toString() === testModuleNode.definitionUri.toString())
        , "The test module is either a declaration module, or the root module of some target node");

    const definitionUri = testModuleNode.definitionUri;

    const runnables = await getNormalizedTestRunnablesInFile(definitionUri);

    await updateFileDefinitionTestModuleByRunnables(testModuleNode, runnables);
}

function categorizeRunnables(runnables: RunnableFacde[]) {
    const testModuelRunnables = runnables.filter(it =>
        it.testKind === NodeKind.TestModule);

    const testRunnables = runnables.filter(it =>
        it.testKind === NodeKind.Test);

    assert(testModuelRunnables.length + testRunnables.length === runnables.length);

    const declarationModuleRunnables = testModuelRunnables.filter(r => r.isTestModuleDeclarationRunnable);
    const fileDefinitionModuleRunnables = testModuelRunnables.filter(r => r.isTestModuleFileDefinitionRunnable);
    const withItemsModuleRunnables = testModuelRunnables.filter(r => r.isTestModuleWithItemsRunnable);

    assert(declarationModuleRunnables.length + fileDefinitionModuleRunnables.length + withItemsModuleRunnables.length === testModuelRunnables.length);
    return {
        testRunnables,
        declarationModuleRunnables,
        fileDefinitionModuleRunnables,
        withItemsModuleRunnables
    };
}

/**
 * If node and runnable is matched, the only possible differnce should be location
 * In the other word, the test item build by the test model could be run through this runnable
 */
function isTestNodeAndRunnableMatched(node: TestLikeNode, runnable: RunnableFacde): node is TestLikeNode {
    return runnable.testPaths.join() === node.testPaths.join();
}

/**
 * Update test module node's children with new fetched runnables
 *
 * It's assumed the runnables are fetched from the same file of the test module node
 *
 * @param parentNode
 * @param runnables
 */
async function updateFileDefinitionTestModuleByRunnables(parentNode: TestModuleNode, runnables: RunnableFacde[]) {
    const { added, deleted, updated } = distinguishChanges(runnables);

    /// updated
    updated.forEach(([testLikeNode, runnable]) => {
        // update the relationship
        // although it should be fine to not update, because we only use runnable later to run/debug
        // and use the old runnable does not influce the args
        runnableByTestModel.set(testLikeNode, runnable);
        // update the location
        updateLocationOfTestLikeByRunnable(testLikeNode, runnable);
    });

    /// deleted
    deleted.forEach(node => {
        assert(node.parent.kind === NodeKind.TestModule);
        node.parent.testChildren.delete(node);
    });

    /// added
    const { declarationModuleRunnables, fileDefinitionModuleRunnables, testRunnables, withItemsModuleRunnables, } = categorizeRunnables(added);

    // Handle fileDefinitionModules
    // Not Handle fileDefinitionModule, we choose to use definition rather then declaration as the presentation of test module
    // which means, when user goto the test, will be rediredct to the declaration rather than some file.

    // Handle testRunnables and test modules which have items, which are in the same test file
    addTestModuleWithItemsRunnablesToTestModule(parentNode, withItemsModuleRunnables);
    addTestModuleWithItemsRunnablesToTestModule(parentNode, testRunnables);

    // Handle declarationModules
    // TODO: maybe concurrent?
    for (const declarationModuleRunnable of declarationModuleRunnables) {
        await addDeclarationModuleRunnableToTestModule(parentNode, declarationModuleRunnable);
    }

    function distinguishChanges(runnables: RunnableFacde[]) {
        const { declarationModuleRunnables, fileDefinitionModuleRunnables, testRunnables, withItemsModuleRunnables } = categorizeRunnables(runnables);

        const finalRunnables = [...declarationModuleRunnables, ...testRunnables, ...withItemsModuleRunnables];

        // get previous children of the test module node
        const childrenOfParentnode = ChildrenCollector.collect(parentNode);
        const childrenInTheSameFile = childrenOfParentnode.filter(node => isTestLikeNodeInTheSameFile(node, parentNode));

        // distinguish update/add/delete nodes
        const updated: [TestLikeNode, RunnableFacde][] = [];
        const added: RunnableFacde[] = [];
        const deleted: Set<TestLikeNode> = new Set(childrenInTheSameFile);

        finalRunnables.forEach(it => {
            const testNode = DummyRootNode.instance.findNearestNodeByRunnable(it);
            assert(isTestLikeNode(testNode));
            if (isTestNodeAndRunnableMatched(testNode, it)) {
                updated.push([testNode, it]);
                deleted.delete(testNode);
            } else {
                added.push(it);
            }
        });

        return {
            updated,
            added,
            deleted,
        };
    }

    function isTestLikeNodeInTheSameFile(a: TestLikeNode, b: TestModuleNode) {
        switch (a.kind) {
            case NodeKind.TestModule:
                return a.declarationInfo.uri.toString() === b.definitionUri.toString();
            case NodeKind.Test:
                return a.location.uri.toString() === b.definitionUri.toString();
        }
    }

    function updateLocationOfTestLikeByRunnable(node: TestLikeNode, runnable: RunnableFacde) {
        switch (node.kind) {
            case NodeKind.TestModule:
                node.declarationInfo = runnable.toTestLocation();
                break;
            case NodeKind.Test:
                node.location = runnable.toTestLocation();
        }
    }

    async function addDeclarationModuleRunnableToTestModule(parentNode: TestModuleNode, declarationModuleRunnable: RunnableFacde) {
        const definition = await getModuleDefinitionLocation(declarationModuleRunnable);

        // Add declarationModule node into the tree
        const testModule = new TestModuleNode(
            parentNode,
            declarationModuleRunnable.testOrSuiteName,
            declarationModuleRunnable.toTestLocation(),
            vscode.Uri.parse(definition.targetUri));
        runnableByTestModel.set(testModule, declarationModuleRunnable);
        parentNode.testChildren.add(testModule);
    }

    // This function will add the descendants of a test module into it
    function addTestModuleWithItemsRunnablesToTestModule(parentNode: TestModuleNode, runnables: RunnableFacde[]) {
        // sort to ensure the parent is added before the chidren
        runnables.sort(RunnableFacde.sortByLabel)
            .forEach(runnable => {
                const parentNode = DummyRootNode.instance.findNearestNodeByRunnable(runnable);
                assert(parentNode.kind === NodeKind.TestModule, "Runable should be inserted into TestModule/Test, we create mock runnable for target/workspace node");
                if (!parentNode.isDummyTestModule()) {
                    assert(parentNode.name === runnable.testPaths[runnable.testPaths.length - 2]);
                }

                switch (runnable.testKind) {
                    case NodeKind.Test:
                        const testNode = new TestNode(parentNode,
                            runnable.toTestLocation(),
                            runnable.testOrSuiteName);
                        runnableByTestModel.set(testNode, runnable);
                        parentNode.testChildren.add(testNode);
                        break;
                    case NodeKind.TestModule:
                        const testModuleNode = new TestModuleNode(
                            parentNode,
                            runnable.testOrSuiteName,
                            runnable.toTestLocation(),
                            runnable.uri,
                        );
                        runnableByTestModel.set(testModuleNode, runnable);
                        parentNode.testChildren.add(testModuleNode);
                        break;
                    default:
                        assertNever(runnable.testKind);
                }
            });
    }
}

// Get all children of a test-like node
class ChildrenCollector extends WorkspacesWalker {
    private constructor(private rootNode: TestLikeNode) {
        super();
    }

    public static collect(node: TestLikeNode) {
        const it = new ChildrenCollector(node);
        it.apply(node);
        return Array.from(it.result);
    }

    private result: Set<TestLikeNode> = new Set();

    protected override visitTestModuleNode(node: TestModuleNode): void {
        if (!(node === this.rootNode)) {
            this.result.add(node);
        }

        super.visitTestModuleNode(node);
    }

    protected override visitTestNode(node: TestNode): void {
        this.result.add(node);

        super.visitTestNode(node);
    }
}

const testItemByTestLike = new Map<TestLikeNode, vscode.TestItem>();
const testModelByTestItem = new WeakMap<vscode.TestItem, Nodes>();
const runnableByTestModel = new WeakMap<TestLikeNode, RunnableFacde>();

export function getTestItemByTestLikeNode(testLikeNode: TestLikeNode): vscode.TestItem {
    const testItem = testItemByTestLike.get(testLikeNode);
    assert(!!testItem);
    return testItem;
}

export function getTestModelByTestItem(testItem: vscode.TestItem): Nodes {
    const testModel = testModelByTestItem.get(testItem);
    assert(!!testModel);
    return testModel;
}

function getRunnableByTestModel(testModel: Nodes): RunnableFacde {
    let testLikeNode: TestLikeNode | undefined;
    switch (testModel.kind) {
        case NodeKind.DummyRoot:
            fail("Never");
        case NodeKind.CargoWorkspace:
            fail("Do not support for now");
        case NodeKind.CargoPackage:
            return createMockPackageRootRunnable(testModel);
        case NodeKind.Target: {
            testLikeNode = testModel.dummyTestModule;
            const runnable = runnableByTestModel.get(testLikeNode);
            assert(!!runnable);
            return runnable;
        }
        case NodeKind.TestModule:
        case NodeKind.Test:
            testLikeNode = testModel;
            const runnable = runnableByTestModel.get(testLikeNode);
            assert(!!runnable);
            return runnable;
        default:
            assertNever(testModel);
    }

    function createMockPackageRootRunnable(packageNode: CargoPackageNode) {
        const packageMockRunnable: ra.Runnable = {
            label: 'test-mod ',
            kind: 'cargo',
            location: {
                targetUri: packageNode.manifestPath.fsPath,
                targetRange: new vscode.Range(0, 0, 0, 0),
                targetSelectionRange: new vscode.Range(0, 0, 0, 0),
            },
            args: {
                workspaceRoot: packageNode.parent.workspaceRoot.fsPath,
                cargoArgs: [
                    "test",
                    "--package",
                    packageNode.name,
                    "--lib",
                    "--bins",
                    "--tests",
                ],
                cargoExtraArgs: [],
                executableArgs: [],
            }
        };

        const packgeRunnable = new RunnableFacde(packageMockRunnable);

        return packgeRunnable;
    }
}

export function getRunnableByTestItem(testItem: vscode.TestItem): RunnableFacde {
    const testModel = getTestModelByTestItem(testItem);
    const runnable = getRunnableByTestModel(testModel);
    return runnable;
}

// Build vscode.TestItem tree
// and bind TestModel and vscode.TestItem
class VscodeTestTreeBuilder extends WorkspacesWalker {
    private static singlton = new VscodeTestTreeBuilder();

    public static buildChildrenFor(node: TestModuleNode) {
        const { singlton } = VscodeTestTreeBuilder;
        // not traversal the node itself
        node.testChildren.forEach(child => {
            singlton.apply(child);
        });
    }

    public static build() {
        const { singlton } = VscodeTestTreeBuilder;
        testItemByTestLike.clear();
        singlton.rootTestItems = [];
        singlton.testItemByNode.clear();
        singlton.apply(DummyRootNode.instance);
        const result = singlton.rootTestItems;
        return result;
    }

    private rootTestItems: vscode.TestItem[] = [];

    private testItemByNode = new Map<Nodes, vscode.TestItem>();

    private addTestItemToParentOrRoot(node: Nodes, testItem: vscode.TestItem) {
        testModelByTestItem.set(testItem, node);

        if (isTestModuleNode(node) || isTestNode(node)) {
            testItemByTestLike.set(node, testItem);
        }

        this.testItemByNode.set(node, testItem);

        const parentTestItem = tryGetParentTestItem.call(this, node);

        if (parentTestItem) {
            parentTestItem.children.add(testItem);
        } else {
            this.rootTestItems.push(testItem);
        }

        function tryGetParentTestItem(this: VscodeTestTreeBuilder, node: Nodes) {
            let curNode = node;
            while (curNode.parent) {
                const candidate = this.testItemByNode.get(curNode.parent);
                if (candidate) {
                    return candidate;
                }
                curNode = curNode.parent;
            }
            return undefined;
        }
    }

    // Need this, for we do not delete workace node unless refetch metadata.
    private isWorkspaceEmptyWithTests(node: CargoWorkspaceNode) {
        return node.members.every(this.isPackageEmptyWithTests);
    }

    // Need this, we do not delete package node unless refetch metadata.
    private isPackageEmptyWithTests(node: CargoPackageNode) {
        return node.targets.size === 0;
    }

    protected override visitCargoWorkspaceNode(node: CargoWorkspaceNode) {
        // if there is only one workspace, do not create a test item node for it
        // Flatten the items
        if (DummyRootNode.instance.roots.length === 1) {
            return super.visitCargoWorkspaceNode(node);
        }

        // if there is no tests in workspace, not create test-item.
        // and not traversal subtree
        if (this.isWorkspaceEmptyWithTests(node)) {
            return;
        }

        const testItem = testController!.createTestItem(node.workspaceRoot.toString(), `$(project)${node.workspaceRoot.fsPath}`, node.manifestPath);
        this.addTestItemToParentOrRoot(node, testItem);

        super.visitCargoWorkspaceNode(node);
    }

    protected override visitCargoPackageNode(node: CargoPackageNode) {
        // if there is only one package, do not create a test item node for it
        // Flatten the items
        if (node.parent.members.length === 1) {
            return super.visitCargoPackageNode(node);
        }

        // if there is no tests in workspace, not create test-item.
        // and not traversal subtree
        if (this.isPackageEmptyWithTests(node)) {
            return;
        }

        const testItem = testController!.createTestItem(node.manifestPath.fsPath, `$(package)${node.name}`, node.manifestPath);
        this.addTestItemToParentOrRoot(node, testItem);

        super.visitCargoPackageNode(node);
    }

    protected override visitTargetNode(node: TargetNode) {
        // if there is only one target, do not create a test item node for it
        // Flatten the items
        if (node.parent.targets.size === 1) {
            return super.visitTargetNode(node);
        }

        let icon: string;
        switch (node.targetKind) {
            case TargetKind.Binary:
                icon = "$(run)";
                break;
            case TargetKind.Library:
                icon = "$(library)";
                break;
            case TargetKind.IntegrationTest:
                icon = "$(beaker)";
                break;
            default:
                assertNever(node.targetKind);
        }

        const testItem = testController!.createTestItem(`${icon}${node.name}`, `${icon}${node.name}`, node.srcPath);
        this.addTestItemToParentOrRoot(node, testItem);

        super.visitTargetNode(node);
    }

    protected override visitTestModuleNode(node: TestModuleNode) {
        if (node.isDummyTestModule()) {
            // not create test item for root test module, which is representated by corresponding target node.
            return super.visitTestModuleNode(node);
        }

        const testItem = testController!.createTestItem(node.name, `$(symbol-module)${node.name}`, node.declarationInfo.uri);
        testItem.range = node.declarationInfo.range;
        const isChildrenFetched = node.testChildren.size !== 0;
        const isDeclarationModule = node.declarationInfo.uri.toString() !== node.definitionUri.toString();

        if (!isChildrenFetched && isDeclarationModule) {
            testItem.canResolveChildren = true;
        }

        this.addTestItemToParentOrRoot(node, testItem);

        super.visitTestModuleNode(node);
    }

    protected override visitTestNode(node: TestNode) {
        const testItem = testController!.createTestItem(node.name, `$(symbol-method)${node.name}`, node.location.uri);
        testItem.range = node.location.range;
        this.addTestItemToParentOrRoot(node, testItem);

        super.visitTestNode(node);
    }
}

async function getModuleDefinitionLocation(runnable: RunnableFacde) {
    assert(runnable.isTestModuleDeclarationRunnable);

    const definitionLocations = await RaApiHelper.moduleDefinition(runnable.origin.location!);

    assert(definitionLocations?.length === 1, "There should always be one and only one module definition for any module declaration.");

    return definitionLocations[0]!; // safe, for we have checked the length
}
