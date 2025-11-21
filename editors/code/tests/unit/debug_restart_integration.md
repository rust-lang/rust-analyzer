# Debug Restart Integration Test Documentation

## Problem Statement
When debugging Rust tests, pressing the "restart" button should recompile the code with any changes before restarting the debug session. However, the behavior of VS Code's debug API has changed across versions:

- **Before**: `onDidTerminateDebugSession` was fired when pressing restart
- **Now**: Only `restart` DAP command is sent, `onDidTerminateDebugSession` is not fired

## Current Solution
We use `DebugAdapterTracker.onWillReceiveMessage` to intercept the `restart` DAP command and trigger recompilation.

## Testing Strategy

### Unit Tests
Located in `debug_restart.test.ts`:
- Verify restart command triggers the handler
- Verify session tracking works correctly
- Verify invalidate request is sent

### Manual Integration Tests
These should be performed when:
1. Upgrading VS Code engine version
2. Upgrading CodeLLDB/debug adapter versions
3. Making changes to debug.ts

#### Test Case 1: Basic Restart
1. Open a Rust project with tests
2. Add a breakpoint in a test
3. Start debugging a test
4. While paused, modify the code (e.g., change a variable value)
5. Save the file
6. Press the restart button (green circular arrow)
7. **Expected**: Compilation runs, new code is used
8. **Verify**: Variable shows new value when stepping through

#### Test Case 2: Multiple Restarts
1. Start debugging a test
2. Modify code and save
3. Press restart
4. Repeat 2-3 times
5. **Expected**: Each restart uses the latest code
6. **Verify**: No stale binaries or cached values

#### Test Case 3: Different Debug Adapters
Test with each supported debug adapter:
- CodeLLDB (vadimcn.vscode-lldb)
- lldb-dap (llvm-vs-code-extensions.lldb-dap)
- C/C++ (ms-vscode.cpptools)
- Native Debug (webfreak.debug)

**Expected**: Restart + recompile works consistently

### Debugging Test Failures

If restart stops working:

1. Check Output panel → "rust-analyzer" for debug logs
2. Look for:
   - "Restart detected, recompiling before restart"
   - "Recompilation complete"
   - "Sent invalidate request to debug adapter"
   
3. Check Terminal for cargo build output

4. Verify DAP messages by adding logging:
   ```typescript
   if (msg.command) {
       log.debug(`DAP message command: ${msg.command}`);
   }
   ```

5. Check VS Code version and debug adapter version compatibility

### Known Issues

#### Issue: Debugger shows old values even after recompilation
**Symptom**: Code recompiles correctly but debugger displays stale values

**Cause**: Debug adapter caches binary/symbols

**Solution**: Use stop + restart approach instead of simple await:
```typescript
// Stop session, recompile, start fresh
vscode.debug.stopDebugging(session).then(async () => {
    await recompileTestFromDebuggingSession(session, ctx);
    await vscode.debug.startDebugging(session.workspaceFolder, session.configuration);
});
```

#### Issue: Restart button stops working
**Symptom**: Clicking restart does nothing or doesn't recompile

**Possible Causes**:
1. VS Code changed the DAP protocol
2. Debug adapter changed how restart is handled
3. `onWillReceiveMessage` signature changed

**Investigation Steps**:
1. Add comprehensive logging to see what messages are received
2. Check VS Code release notes for debug API changes
3. Check debug adapter changelog
4. Test with older VS Code version to isolate the issue

### Version Compatibility Matrix

| VS Code | CodeLLDB | Behavior | Works? |
|---------|----------|----------|--------|
| 1.95+   | 1.10+    | restart command via DAP tracker | ✅ |
| 1.94    | 1.9      | onDidTerminateDebugSession | ✅ (old approach) |

## Future Improvements

1. **Add telemetry**: Track restart success/failure rates
2. **Add user notification**: Show when recompilation is happening
3. **Add configuration option**: Allow users to disable auto-recompile
4. **Detect compilation errors**: Handle failed builds gracefully
