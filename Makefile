clean: fmt
	rm -rf target/wasm32-unknown-unknown/release

build:
	make clean
	stellar contract build
	stellar contract optimize --wasm target/wasm32-unknown-unknown/release/kale_sc.wasm

upload:
	make build
	stellar contract upload --wasm target/wasm32-unknown-unknown/release/kale_sc.optimized.wasm --network testnet --source default

deploy:
	make build
	stellar contract deploy --wasm target/wasm32-unknown-unknown/release/kale_sc.optimized.wasm --network testnet --source default -- --farmer default --asset CDQKZ76ZS7LYDOZ2E7OG5LUJEWDDUNYBVYRJTBJK6645DZBNJWA7DXCR

fmt:
	cargo fmt --all