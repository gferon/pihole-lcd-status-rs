build:
	cargo build --target=arm-unknown-linux-gnueabihf --release

deploy: build
	scp -C ../target/arm-unknown-linux-gnueabihf/debug/winterr pizero:.

run: deploy
	ssh pizero ./winterr