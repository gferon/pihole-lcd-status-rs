build:
	cargo build --target=armv7-unknown-linux-gnueabihf --release

setup:
	ssh -q pi mkdir -p .config/systemd/user
	scp -qC dist/pihole-lcd-status.service pi:.config/systemd/user/pihole-lcd-status.service
	ssh -q pi systemctl --user daemon-reload

deploy: build
	ssh pi "systemctl stop pihole-lcd-status"
	scp -qC ../target/armv7-unknown-linux-gnueabihf/release/pihole-lcd-status pi:.
	ssh pi "systemctl start pihole-lcd-status"