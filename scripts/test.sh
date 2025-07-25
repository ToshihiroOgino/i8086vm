#!/bin/bash
TARGET=$1
FILES=$(find ./setuptools/tests -maxdepth 1 -name '*.c' -type f | xargs -I x basename x)

function is_skip() {
    local file="$1"
    # ignore file that starts with a dot
    if [[ "$file" == .* ]]; then
        return 1
    fi
    return 0
}

mkdir -p ./test/binary
mkdir -p ./test/mmvm
mkdir -p ./test/res

# Suppress warnings from the Rust compiler
export RUSTFLAGS="-Awarnings"

if [ -z "$TARGET" ] || [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    echo "Usage:"
    echo "  $0 <target>"
    echo "  $0 --list(-l)"
    echo "  $0 --all(-a)"
    exit 0
elif [ "$1" = "-l" ] || [ "$1" = "--list" ]; then
    echo "Usage: $0 <target>"
    echo "available targets:"
    for FILE in $(sort <<<"$FILES"); do
        if ! is_skip "$FILE"; then
            continue
        fi
        echo "  - $FILE"
    done
    exit 0
elif [ "$1" = "--all" ] || [ "$1" = "-a" ]; then
    for FILE in $(sort <<<"$FILES"); do
        if ! is_skip "$FILE"; then
            continue
        fi
        TARGET=${FILE}
        echo "Testing $TARGET"
        ./scripts/test.sh $TARGET
    done
    exit 0
fi

BIN_FILE="./test/binary/$TARGET.out"
/usr/local/core/bin/m2cc ./setuptools/tests/$TARGET -o $BIN_FILE 2>/dev/null

EXPECTED_FILE="./test/mmvm/$TARGET.out.txt"
/usr/local/core/bin/mmvm -d $BIN_FILE 2>$EXPECTED_FILE

RES_FILE="./test/res/$TARGET.out.txt"
cargo run --quiet -- -d $BIN_FILE >$RES_FILE

diff -i $RES_FILE $EXPECTED_FILE
