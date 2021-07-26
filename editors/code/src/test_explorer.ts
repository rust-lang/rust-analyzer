import * as vscode from 'vscode';
import * as path from 'path';
import { array, func } from 'vscode-languageclient/lib/utils/is';
import { strict } from 'assert';
import { Func } from 'mocha';


const iconsRootPath = path.join(path.dirname(__dirname), '..', 'resources', 'icons'); 

function getIconUri(iconName: string, theme: string): vscode.Uri {
	return vscode.Uri.file(path.join(iconsRootPath, theme, `${iconName}.svg`));
}

/// Runnable.

type Node = Workspace | Crate | Module | Function;

enum NodeKind {
    Workspace,
    Crate,
    Module,
    Function,
}

type Session = Iterable<Workspace>;

interface Workspace {
    kind: NodeKind.Workspace,
    id: string, 
    crates: Crate[],
    location: string,
}

interface Crate {
    kind: NodeKind.Crate,
    id: string, 
    name: string,
    modules: Module[], 
    location: string,
}

interface Module {
    kind: NodeKind.Module,
    id: string, 
    name: string,
    modules?: Module[], 
    targets?: Function[],
    location: string,
}

enum TestKind {
    Unit,
    Bench,
}

interface Function {
    kind: NodeKind.Function,
    id: string, 
    name: string,
    location: string,
    testKind: TestKind,
}

/// The view synchronized with RA data by `DeltaUpdate`'s. The update is an array   
/// of elementary actions called a `Patch`. After applying an update to the tree 
/// it will become synchronized.

type DeltaUpdate = Iterable<Patch>;

type Patch = Delete | Update | Create;

enum PatchKind {
    Delete = "DELETE",
    Update = "UPDATE",
    Create = "CREATE"
}

interface Delete {
    kind: PatchKind.Delete,
    targetId: string,
} 

interface Update {
    kind: PatchKind.Update,
    targetId: string,
    payload: {
        name?: string,
        location?: string,
        testKind?: TestKind,
    },
}

interface Create {
    kind: PatchKind.Create,
    targetId: string,
    payload: Node,
}

class Workspace extends vscode.TreeItem {  
    description = this.location;

    iconPath = {
        light: getIconUri('squares', 'dark'),
        dark: getIconUri('squares', 'dark'),
    };

    getChildren(): Crate[]{
        return this.crates;
    }
}

class Crate extends vscode.TreeItem {
    constructor(
        id: string, 
        name: string,
        modules: Module[], 
        location: string,
    ) {
        super(name, vscode.TreeItemCollapsibleState.Collapsed);
        this.location = location;
        this.id = id;
        this.modules = modules;
    }
    
    description = this.location;

    iconPath = {
        light: getIconUri('squares', 'dark'),
        dark: getIconUri('squares', 'dark'),
    };

    getChildren(): Module[] {
        return this.modules;
    }
}

class Module extends vscode.TreeItem {
    constructor(
        id: string, 
        name: string,
        location: string,
        modules?: Module[], 
        targets?: Function[],
    ) {
        super(name, vscode.TreeItemCollapsibleState.Collapsed);
        this.location = location;
        this.id = id;
    }   

    description = this.location;

    iconPath = {
        light: getIconUri('squares', 'dark'),
        dark: getIconUri('squares', 'dark'),
    };

    getChildren(): (Function | Module)[] {
        var res: (Function | Module)[] = [];
        if(this.targets != undefined) {
            res.concat(this.targets);
        }
        if(this.modules != undefined) {
            res.concat(this.modules);
        }
        return res;
    }
}

class Function extends vscode.TreeItem {
    constructor(
        id: string, 
        name: string,
        location: string,
        kind: TestKind,
    ) {
        super(name, vscode.TreeItemCollapsibleState.None);
        this.location = location;
        this.id = id;
        
        switch(kind) {
            case TestKind.Bench: {
                this.iconPath = {
                    light: getIconUri('accelerator', 'dark'),
                    dark: getIconUri('accelerator', 'dark'),
                };
                break;
            }
            case TestKind.Unit: {
                this.iconPath = {
                    light: getIconUri('test_sheet', 'dark'),
                    dark: getIconUri('test_sheet', 'dark'),
                };
                break;
            }
        }
    }
       
    description = this.location;

    getChildren(): null {
        return null;
    }   
}

function bfs(root: Node, process: (parentField: Node[], node: Node) => void) {
    let queue: Array<Node> = [root];
    while(queue.length != 0) {
        let current = queue.pop();
        //@ts-ignore
        process(current);
        //@ts-ignore
        current.getChildren();
    }
}

export class RunnableDataProvider implements vscode.TreeDataProvider<Node> {
    private _onDidChangeTreeData: vscode.EventEmitter<Node | undefined> = new vscode.EventEmitter<Node | undefined>()
    readonly onDidChangeTreeData: vscode.Event<Node | undefined> = this._onDidChangeTreeData.event;

    getChildren(element?: Node): vscode.ProviderResult<Node[]> {
        if(element == undefined) {
            return Promise.resolve([]); 
        }
        
        return element.getChildren();
    }
    
    getTreeItem(element: Node): Node {
        return element;
    }   
}

export class RunnableView {
    private dataProvider: RunnableDataProvider;
    private tree: Node;

    constructor(context: vscode.ExtensionContext) {
        this.dataProvider = new RunnableDataProvider()
    }
  
    handleCreate(node: Node, patch: Create) {
        switch(node.kind) {
            case NodeKind.Workspace: {
                if (patch.payload.kind != NodeKind.Crate) {
                    throw Error(`${patch.payload.kind} cant't be payload for ${NodeKind.Workspace} target`);
                }
                node.crates.push(patch.payload);
            }
            break;
            case NodeKind.Crate: {
                if (patch.payload.kind != NodeKind.Module) {
                    throw Error(`${patch.payload.kind} cant't be payload for ${NodeKind.Crate} target`);
                }
                node.modules.push(patch.payload);
            }
            break;
            case NodeKind.Module: {
                if (patch.payload.kind == NodeKind.Module) {
                    if(node.modules == undefined) {
                        node.modules = [];
                    }
                    node.modules!.push(patch.payload);
                } else if (patch.payload.kind == NodeKind.Function) {
                    if(node.modules == undefined) {
                        node.modules = [];
                    }
                    node.targets!.push(patch.payload);
                } else {
                    throw Error(`${patch.payload.kind} cant't be payload for ${NodeKind.Module} target`);
                }
            }
            break;
            case NodeKind.Function: {
                throw Error("Function can't be a target for Create's patch");
            }
        }
    }

    handleDelete(node: Node, parentField: Array<Node>) {
        const index = parentField.indexOf(node);
        parentField.splice(index, 1);
    }

    handleUpdate(node: Node, patch: Update) {
        switch(node.kind) {
            case NodeKind.Workspace: {
                node.location = patch.payload.location!;
            }
            break;
            case NodeKind.Crate: {
                node.location = patch.payload.location!; 
                node.name = patch.payload.name!;
            }
            break;
            case NodeKind.Module: {
                node.name = patch.payload.name!;
                node.location = patch.payload.location!;
            }
            break;
            case NodeKind.Function: {
                node.name = patch.payload.name!;
                node.location = patch.payload.location!;
                node.testKind = patch.payload.testKind!;
            }
            break;
        }
    }

    public applyUpdate(update: DeltaUpdate) {
        for (let patch of update) {
            bfs(this.tree, (parentField, node) => {
                if(node.id == patch.targetId) {
                    switch(patch.kind) {
                        case PatchKind.Create: {
                            this.handleCreate(node, patch);
                        }
                        break;
                        case PatchKind.Delete: {
                            this.handleDelete(node, parentField)
                        }
                        break;
                        case PatchKind.Update: {
                            this.handleUpdate(node, patch)
                        }
                        break;
                    }
                }
            })
        }  
    }
}

export class TestView {
    
    constructor() {
        let controller = vscode.tests.createTestController();
    }
}

// function runExecutable() {
//     // TODO: implement 
// }

// function goToDefinition() {
//     vscode.workspace.openTextDocument().then((document)=>{
//         vscode.window.activeTextEditor?.revealRange();    
//     });
// } 
// Для Workspace и Crate show in explorer

// TODO: возможность создавать run lists
// TODO: Run History




