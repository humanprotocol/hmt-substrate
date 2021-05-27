#!/bin/bash

steps=20
repeat=20
output=./runtime/src/weights/
chain=dev
statemintChain=statemint-dev
pallets=(
	pallet_escrow
	pallet_kvstore
)

# build the binary with runtime benchmarks included
cargo build --manifest-path node/Cargo.toml --release --features=runtime-benchmarks

# run the benchmarks for all the pallets
for p in ${pallets[@]}
do
	./target/release/node-template benchmark \
		--chain $chain \
		--execution wasm \
		--wasm-execution compiled \
		--pallet $p  \
		--extrinsic '*' \
		--steps $steps  \
		--repeat $repeat \
		--raw  \
		--output
	
	mv "$p.rs" $output

done