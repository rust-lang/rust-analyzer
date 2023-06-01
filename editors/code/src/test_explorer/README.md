## How it works

### Glossary
Runnable: Rust Analyzer has an internal structure called "Runnable", which is used to debug/run, which you might already be familar with.
TestItem: This is the structure used by vscode, and it's the surface of VSCode and RA.
TestModelNodes: This is a very easy AST, help to store meta info of tests and structure.

### Basic
Bascially, we maintain TestModel tree and build test items based on TestModel tree and runnables.



## Issues
There are many strategies about when to send what requests.

Like the laziness is a big choice. When would you like to load how many tests?

An obvious choice to to load all tests for all projects at the first time, then update the changed files.

Another choice is only to load test cases laziness. Only when we open a file or click expand button of the case in test explorer, we load itself and its parents if they are not loaded yet. (this is what's used now, but this might introduce more bugs! Please submit an issue if you met it.)

1. Where should user go when they click "open file" for test module, definition or the declaration?

For now, I choose declaration
``` rs
//// mod.rs
mod foo;  // <-- user will be redirect to here

//// foo.rs
// some code(first line) // rather than here
// some code
```

Because most people know F12(goto implementation), and less people know "locate parent module" command.

2. How to know whether a test case start? When run the whole test suite, how to know the test case in it is queued or started?

Because the output is only text(some other framework might provide a server), we could only analytics the output. However, this is unstable and buggy in nature. And we could not always get what we want. In the worst case, we could only guess.

For example
```
--- Workspace
|  //omit cargo file
|-package1
|    |  // omit cargo file
|    |-tests
|        |-foo-bar.rs
|
|
|-package2
|    |  // omit cargo file
|    |-tests
|        |-foo-bar.rs
```
This is valid, however, the output will be somthing like
```
     Running tests/foo-bar.rs (target/debug/deps/foo_bar-b2e07b357bb81962)

running 1 test
test foo1 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/foo-bar.rs (target/debug/deps/foo_bar-ce4c61ef5dd225ce)

running 1 test
test foo2 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
We could not distinguilish which target is executed exactly. The best thing we could do is to guess by the test path(in this example, they are "foo1" and "foo2")

But the guess logic is not implemented yet :P. Instead, we not allow to run test on workspace level.

3. For cargo, there is no way to match test mod exactly, let's say you have tests
``` rs
mod1::mod2::mod3::test1

mod2::mod3::test2

mod2::mod3::test3
```

Then, you want to test all cases under mod2::mod3, but sadly, test1 will be matched too. This will rarely happens in a real repo, but it should be a flaw.

And you could even declare such situation
``` rs
mod1::foo(this is a test module)

mod1::foo(this is a test case)
```

When you want to run `mod1::foo`(module), the cause will be matched too.

- Maybe we could add "::" at the end if it's a test module

4. Altough in the design, the `path` attribute is considered, it will make things much more complex, let skip it for the first PR.

5. How to make sure ra is updated before the request?

6. The error message shown on the test rather than the line.
    - enhance the analyze

7. User could only choose one test case to run
    - Maybe filter could help
    - But it seems we could never run differnt target

8. As mentioned in 2 point, "run all" does not work for workspace level for now.

9. Debug will not update the state of test item.(could provide better experience for Linux)


### mermaid graph

Use https://mermaid.live/ or VSCode plugin to show the flow

Init

``` mermaid
sequenceDiagram
    participant U as User
    participant VSC as VSCode
    participant C as VSCode extension
    participant RA as Rust Analyzer

    U->>+VSC: Open testing explorer
    VSC->>+C: resolver sends init request
    C->>+RA: get cargo project info
    RA->>-C: back cargo project info
    C->>C: construct Ideal tree about workspace and package part
    loop until all targets are got
    C->>+RA: request test infos for target root files (repeat many times)
    RA->>-C: request test infos for target root files
    end
    C->>C: construct Ideal tree about target root files part
    C->>C: construct TestItem tree By Ideal tree
    C->>-VSC: VSCode got the Test Item tree
    VSC->>-U: Render Test Explorer
```

Change avtive file
``` mermaid
sequenceDiagram
    participant U as User
    participant VSC as VSCode
    participant C as VSCode extension
    participant RA as Rust Analyzer

    U->>+VSC: Change active document
    VSC->>+C: trigger event
    C->>C: check whether the file is already loaded
    alt is already loaded
    C->>VSC: nothing changed
    VSC->>U: nothing changed
    else file is not loaed
    Note over C: Load the file, but we might need to load its parent
    loop until the module is loaded
    C->>C: find nearest parent of the file module in ideal tree
    C->>RA: get module info of the file
    RA->>C: return module info of the file
    C->>C: add new nodes to ideal tree
    end
    C->>C: construct TestItem tree By Ideal tree
    C->>-VSC: VSCode got the Test Item tree
    VSC->>-U: Render Test Explorer
    end
```

Add rust file
Change rust file
Delete rust file
``` mermaid
sequenceDiagram
    participant U as User
    participant VSC as VSCode
    participant C as VSCode extension
    participant RA as Rust Analyzer

    U->>+VSC: Add/Delete/Change file
    VSC->>+C: trigger event
    C->>RA: request test info in file
    RA->>C: back test info in file
    C->>C: update ideal tree
    Note over C: This is different for different operations
    C->>C: construct TestItem tree By Ideal tree
    C->>-VSC: VSCode got the Test Item tree
    VSC->>-U: Render Test Explorer
```


Debug
``` mermaid
sequenceDiagram
    participant U as User
    participant VSC as VSCode
    participant C as VSCode extension
    participant LLDB as LLDB extension

    U->>+VSC: Click run/debug button
    VSC->>+C: testing API trigger event
    C->>C: compute VSCode Debug configuration(LLDB)
    C->>C: Set test status for test items
    C->>VSCode: Start debug
    VSCode->>LLDB: Debugger protocal
    LLDB->>VSCode: Debugger protocal
    VSCode->>C: debug session
    C->>C: Analytics test output from rustc and update status of test items
    C->>C: Attach info to test items(if any)
    C->>-VSC: return
    VSC->>-U: return
```

Run
``` mermaid
sequenceDiagram
    participant U as User
    participant VSC as VSCode
    participant C as VSCode extension
    participant Cargo as Cargo

    U->>+VSC: Click run/debug button
    VSC->>+C: testing API trigger event
    C->>C: compute Cargo command
    C->>C: Set test status for test items
    C->>Cargo: execute commands
    Cargo->>C: run tests
    C->>C: Analytics test output from rustc and update status of test items
    C->>C: Attach info to test items(if any)
    C->>-VSC: return
    VSC->>-U: return
```
