#!/bin/bash

set -e

for f in examples/*.ins
do
    dir=$(dirname $f)
    file=$(basename $f .ins)

    echo Starting $file

    ./insc_jvm $f
    java -cp "$dir" "$file" > "$dir/$file.result.jvm"
    diff "$dir/$file.output" "$dir/$file.result.jvm"

    ./insc_llvm $f
    lli "$dir/$file.bc" > "$dir/$file.result.llvm"
    diff "$dir/$file.output" "$dir/$file.result.llvm"

    echo $file OK
done
