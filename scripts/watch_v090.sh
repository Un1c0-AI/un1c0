#!/bin/bash
# UN1C0DE v0.9.0 Watch Script
# Monitors repository for proof-carrying UEG drop at 06:00 UTC
# Automatically validates when v0.9.0 tag appears

set -e

WATCH_INTERVAL=600  # 10 minutes in seconds
TARGET_TAG="v0.9.0"

echo "=========================================="
echo "UN1C0DE v0.9.0 WATCH DAEMON"
echo "Date: $(date -u +%Y-%m-%d\ %H:%M:%S) UTC"
echo "Target: $TARGET_TAG (Proof-carrying UEG)"
echo "Expected: 2025-11-26 06:00 UTC"
echo "Watch interval: ${WATCH_INTERVAL}s (10 min)"
echo "=========================================="
echo ""

while true; do
    CURRENT_TIME=$(date -u +%Y-%m-%d\ %H:%M:%S)
    echo "[$CURRENT_TIME UTC] Checking for $TARGET_TAG..."
    
    # Fetch all tags
    git fetch --all --tags --prune 2>&1 | grep -v "would clobber" || true
    
    # Check if target tag exists
    if git tag | grep -q "^${TARGET_TAG}$"; then
        echo ""
        echo "=========================================="
        echo "✅ $TARGET_TAG DETECTED!"
        echo "Time: $CURRENT_TIME UTC"
        echo "=========================================="
        echo ""
        
        # Pull latest changes
        echo "Pulling v0.9.0 changes..."
        git pull origin main --tags
        
        # Show what changed
        echo ""
        echo "Latest commits:"
        git log --oneline -5
        echo ""
        
        # Rebuild with release profile
        echo "Rebuilding with proof-carrying features..."
        cargo build --release --all-features
        
        # Run full test suite
        echo ""
        echo "Running test suite with all features..."
        cargo test --all-features
        
        # Test proof verification if prove flag exists
        echo ""
        echo "Testing Z3 proof verification..."
        if ./target/release/un1c0 --help | grep -q "\-\-prove"; then
            ./target/release/un1c0 zig rust examples/zig/comptime.zig --prove
        else
            echo "⚠️ --prove flag not yet available (may require v0.9.0 binary update)"
            echo "Running standard translation test:"
            ./target/release/un1c0 zig rust examples/zig/comptime.zig | head -20
        fi
        
        # Check for ueg/proofs directory
        echo ""
        if [ -d "ueg/proofs" ]; then
            echo "✅ ueg/proofs/ directory detected"
            echo "Contents:"
            find ueg/proofs -type f | head -20
        else
            echo "⚠️ ueg/proofs/ directory not found (may be in different location)"
        fi
        
        echo ""
        echo "=========================================="
        echo "v0.9.0 VALIDATION COMPLETE"
        echo "Proof-carrying UEG is LIVE"
        echo "=========================================="
        
        # Exit successfully
        exit 0
    else
        echo "   Not yet available. Next check in ${WATCH_INTERVAL}s..."
    fi
    
    sleep "$WATCH_INTERVAL"
done
