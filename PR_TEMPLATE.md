internal: Improve memory stability and concurrency performance

This PR implements performance optimizations focusing on preventing OOM in large projects and reducing lock contention.

## Changes

### Channel Safety (18 channels)
Convert unbounded channels to bounded to prevent memory exhaustion:
- `prime_caches.rs`: 5 channels → bounded(threads*2)
- `global_state.rs`: 7 channels → bounded with thread-based capacity
- `vfs-notify/src/lib.rs`: 4 channels → bounded(16-256)
- `flycheck.rs`: 2 channels → bounded(16-256)

### Concurrency (2 sites)
Replace `Mutex<HashMap>` with `DashMap` for better parallelism:
- `ProcMacroSrv::expanders`: concurrent proc-macro expander access
- `semantic_tokens_cache`: concurrent semantic token caching

### Memory Optimization
- Increase Salsa LRU capacities: FILE_TEXT 16→32, PARSE 128→256, BORROWCK 2024→4096
- Pre-allocate `Vec` capacity in hot paths: autoderef, attrs, lower, infer

### Dependencies
- Add `dashmap` (workspace) for concurrent maps

## Testing
- `cargo test` passes
- `cargo clippy` passes
- `analysis-stats` verified on rust-analyzer codebase

## AI Disclosure
AI tools (Claude) were used to assist with code analysis and implementation. All changes have been reviewed.
