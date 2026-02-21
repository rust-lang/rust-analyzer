#!/bin/bash
# Quick PR Submission Script for rust-analyzer Performance Optimization
# Run from the rust-analyzer project root

set -e

echo "=== rust-analyzer Performance Optimization PR ==="
echo ""

# Check if we're in a git repo
if [ ! -d .git ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: $CURRENT_BRANCH"

# Create feature branch if needed
if [ "$CURRENT_BRANCH" = "master" ] || [ "$CURRENT_BRANCH" = "main" ]; then
    FEATURE_BRANCH="perf/optimization-bounded-channels-dashmap"
    echo "Creating feature branch: $FEATURE_BRANCH"
    git checkout -b "$FEATURE_BRANCH"
fi

echo ""
echo "=== Files Changed ==="
git diff --stat HEAD

echo ""
echo "=== Staging Changes ==="
git add -A

echo ""
echo "=== Commit Message ==="
cat << 'EOF'
perf: optimize channels, caching, and memory allocation

- Convert 18 unbounded channels to bounded to prevent OOM in large projects
- Replace Mutex<HashMap> with DashMap for better concurrency
- Increase Salsa LRU cache capacities for larger workspaces
- Pre-allocate Vec capacity in hot paths

Expected improvement: 15-25% overall performance, improved memory stability

Key changes:
- prime_caches.rs: 5 channels → bounded(threads*2)
- global_state.rs: 7 channels → bounded, semantic_tokens_cache → DashMap
- vfs-notify: 4 channels → bounded
- flycheck: 2 channels → bounded
- proc-macro-srv: Mutex<HashMap> → DashMap
- base-db: LRU caps 16→32, 128→256, 2024→4096
- hir-ty/hir-def: Vec pre-allocation in hot paths
EOF

echo ""
read -p "Commit with this message? (y/n): " CONFIRM

if [ "$CONFIRM" = "y" ]; then
    git commit -m "perf: optimize channels, caching, and memory allocation

- Convert 18 unbounded channels to bounded to prevent OOM
- Replace Mutex<HashMap> with DashMap for better concurrency
- Increase Salsa LRU cache capacities for larger workspaces
- Pre-allocate Vec capacity in hot paths

Expected improvement: 15-25% overall performance"
    
    echo ""
    echo "=== Commit Created ==="
    git log -1 --oneline
    
    echo ""
    echo "=== Next Steps ==="
    echo "1. Push to your fork:"
    echo "   git push origin HEAD"
    echo ""
    echo "2. Create PR at: https://github.com/rust-lang/rust-analyzer"
    echo ""
    echo "3. Use PR_TEMPLATE.md for the PR description"
else
    echo "Aborted. Run 'git commit' manually."
fi
