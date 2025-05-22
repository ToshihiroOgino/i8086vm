#!/bin/bash
TARGET=$1
if [ -z "$TARGET" ]; then
    echo "Usage: $0 <target>"
    exit 0
elif [ "$1" = "-h" ]; then
    echo "Usage: $0 <target>"
    echo "available targets:"
    FILES=$(find ./setuptools/tests -maxdepth 1 -name '*.c' -type f | xargs -I x basename x)
    for FILE in $(sort <<< "$FILES"); do
        echo "  - $FILE"
    done
    exit 0
fi

BIN_FILE="./test/binary/$TARGET.out"
if [ -f $BIN_FILE ]; then
    rm $BIN_FILE
fi
/usr/local/core/bin/m2cc -.o ./setuptools/tests/$TARGET -o $BIN_FILE

EXPECTED_FILE="./test/mmvm/$TARGET.out.txt"
if [ -f $EXPECTED_FILE ]; then
    rm $EXPECTED_FILE
fi
/usr/local/core/bin/mmvm -d $BIN_FILE 2>$EXPECTED_FILE

RES_FILE="./test/res/$TARGET.out.txt"
if [ -f $RES_FILE ]; then
    rm $RES_FILE
fi
cargo run -- -d $BIN_FILE >>$RES_FILE

diff -i $RES_FILE $EXPECTED_FILE
