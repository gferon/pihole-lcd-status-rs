build:
	cargo build --target=arm-unknown-linux-gnueabihf --release

setup:
	ssh -q pizero mkdir -p .config/systemd/user
	scp -qC dist/winterr.service pizero:.config/systemd/user/winterr.service
	ssh -q pizero systemctl --user daemon-reload

deploy: build
	ssh -q pizero "systemctl --user stop winterr"
	scp -C ../target/arm-unknown-linux-gnueabihf/release/winterr pizero:.

run: deploy
	ssh -q pizero "systemctl --user start winterr"