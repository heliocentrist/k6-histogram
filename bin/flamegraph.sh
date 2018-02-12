#!/usr/bin/env bash

cargo build --release

sudo dtrace -c './target/release/csv_play' -o out.stacks -n 'profile-997 /execname == "csv_play"/ { @[ustack(100)] = count(); }'

../FlameGraph/stackcollapse.pl ./out.stacks | ../FlameGraph/flamegraph.pl > mygraph.svg