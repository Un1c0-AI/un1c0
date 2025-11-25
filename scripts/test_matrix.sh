#!/bin/bash
# UN1C0DE Translation Matrix E2E Test
# Tests all 8x8 = 64 translation paths
# Exit code 0 = 100% success, non-zero = failure count

set -e

BINARY="./target/release/un1c0"
TEMP_DIR="/tmp/un1c0_matrix_test"
mkdir -p "$TEMP_DIR"

LANGS=("python" "solidity" "go" "move" "typescript" "cobol" "swift" "zig")
TOTAL=0
PASSED=0
FAILED=0

echo "=========================================="
echo "UN1C0DE TRANSLATION MATRIX E2E TEST"
echo "Date: $(date -u +%Y-%m-%d\ %H:%M:%S) UTC"
echo "Testing 8x8 = 64 translation paths"
echo "=========================================="
echo ""

# Test each source -> target combination
for SRC in "${LANGS[@]}"; do
    for TGT in "${LANGS[@]}"; do
        TOTAL=$((TOTAL + 1))
        
        # Find example file for source language
        EXAMPLE=""
        case "$SRC" in
            python)     EXAMPLE="examples/python/fib.py" ;;
            solidity)   EXAMPLE="examples/solidity/ERC20.sol" ;;
            go)         EXAMPLE="examples/go/real.go" ;;
            move)       EXAMPLE="examples/move/Token.move" ;;
            typescript) EXAMPLE="examples/ts/component.tsx" ;;
            cobol)      EXAMPLE="examples/cobol/bank_transaction.cbl" ;;
            swift)      EXAMPLE="examples/swift/ViewModel.swift" ;;
            zig)        EXAMPLE="examples/zig/comptime.zig" ;;
        esac
        
        # Skip if example doesn't exist
        if [[ ! -f "$EXAMPLE" ]]; then
            echo "[$TOTAL/64] SKIP: $SRC → $TGT (no example file)"
            continue
        fi
        
        # Run translation
        OUTPUT_FILE="$TEMP_DIR/${SRC}_to_${TGT}.out"
        if timeout 5s "$BINARY" "$SRC" "$TGT" "$EXAMPLE" > "$OUTPUT_FILE" 2>&1; then
            # Check output is non-empty
            if [[ -s "$OUTPUT_FILE" ]]; then
                PASSED=$((PASSED + 1))
                echo "[$TOTAL/64] ✓ PASS: $SRC → $TGT"
            else
                FAILED=$((FAILED + 1))
                echo "[$TOTAL/64] ✗ FAIL: $SRC → $TGT (empty output)"
            fi
        else
            FAILED=$((FAILED + 1))
            echo "[$TOTAL/64] ✗ FAIL: $SRC → $TGT (translation error)"
        fi
    done
done

echo ""
echo "=========================================="
echo "MATRIX TEST RESULTS"
echo "=========================================="
echo "Total paths tested: $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", ($PASSED/$TOTAL)*100}")
echo "Success rate: $SUCCESS_RATE%"
echo "=========================================="

# Cleanup
rm -rf "$TEMP_DIR"

# Exit with failure count
exit $FAILED
