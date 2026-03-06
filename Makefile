build:
	stellar contract build
	stellar contract optimize --wasm target/wasm32v1-none/release/kale_sc.wasm

upload:
	make build
	stellar contract upload --wasm target/wasm32v1-none/release/kale_sc.optimized.wasm --network testnet --source default

deploy:
	make upload
	stellar contract deploy --wasm target/wasm32v1-none/release/kale_sc.optimized.wasm --network testnet --source default --salt 0001 -- --farmer default --asset CAAVU2UQJLMZ3GUZFM56KVNHLPA3ZSSNR4VP2U53YBXFD2GI3QLIVHZZ

fmt:
	cargo fmt --all