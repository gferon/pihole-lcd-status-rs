build:
	cargo build --target=armv7-unknown-linux-gnueabihf --release

deploy: build
	scp -C ../target/armv7-unknown-linux-gnueabihf/release/pihole-lcd-status pi:.

run: deploy
	ssh pi ./pihole-lcd-status