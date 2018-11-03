build:
	cargo build --target=armv7-unknown-linux-gnueabihf --release

deploy: build
	ssh pi "systemctl stop pihole-lcd-status"
	scp -qC ../target/armv7-unknown-linux-gnueabihf/release/pihole-lcd-status pi:.
	scp -qC dist/pihole-lcd-status.service pi:.config/system.d/
	ssh pi "systemctl start pihole-lcd-status"