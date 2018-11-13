build:
	cargo +nightly build --target=arm-unknown-linux-gnueabihf --release

setup: build
	rsync -az dist/ pi0:.config/systemd/user/
	rsync -az dist/ pi1:.config/systemd/user/
	ssh -q pi0 "\
		systemctl --user daemon-reload && \
		systemctl --user enable weatherr.service winterr.service weatherr.timer winterr.timer && \
		systemctl --user start weatherr.timer winterr.timer && \
		systemctl --user list-timers \
	"

deploy: setup
	rsync -z ../target/arm-unknown-linux-gnueabihf/release/*rr pi0:.
	rsync -z ../target/arm-unknown-linux-gnueabihf/release/*rr pi1:.

run: deploy
	ssh -q pi0 "systemctl --user start winterr"