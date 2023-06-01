import type * as vscode from "vscode";
import { testController } from ".";

/**
 * General helper functions for VSCode TestItemController
 */
export abstract class TestItemControllerHelper {
    /**
     *
     * @param cb Stop serach the current subtree if returning non-falsy value. But the rest subtree will continute to search.
     * @param root
     */
    static visitTestItemTreePreOrder(
        cb: (item: vscode.TestItem, collection: vscode.TestItemCollection) => any,
        root: vscode.TestItemCollection = testController!.items,
        exitCb?: (item: vscode.TestItem, collection: vscode.TestItemCollection) => any
    ) {
        root.forEach((item, collection) => {
            const res = cb(item, collection);
            if (res) {
                exitCb?.(item, collection);
                return;
            }
            TestItemControllerHelper.visitTestItemTreePreOrder(cb, item.children, exitCb);
            exitCb?.(item, collection);
        });
    }
}
