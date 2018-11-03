build:
	cargo build --target=armv7-unknown-linux-gnueabihf --release

deploy: build
	ssh pi "systemctl stop pihole-lcd-status"
	scp -C ../target/armv7-unknown-linux-gnueabihf/release/pihole-lcd-status pi:.
	scp -C dist/pihole-lcd-status.service pi:.config/system.d/

run: deploy
	ssh pi "systemctl start pihole-lcd-status"