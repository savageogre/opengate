SOURCE = opengate
RDIR = target/release/
OPENGATE = $(RDIR)opengate

all: lint opengate

i: lint opengate install

install-ubuntu-flac-deps:
	sudo apt update
	sudo apt install libflac-dev -y

DEPRECATED-install-ubuntu-tts-deps:
	sudo apt update
	# libssl error
	sudo apt install pkg-config libssl-dev -y
	# stddef.h error
	sudo apt install build-essential clang libclang-dev -y
	# cmake error
	sudo apt install cmake -y
	# compiling piper-rs fork
	sudo apt install libasound2-dev libespeak-ng-dev -y

opengate-flac:
	cargo build --release --features flac
	cargo build --release --features flac --bin opengate
	cargo build --release --features flac --bin opengate-tts

opengate:
	cargo build --release
	cargo build --release --bin opengate
	cargo build --release --bin opengate-tts

install-flac: opengate-flac
	cargo install --path . --force
	cargo install --path . --force --bin opengate
	cargo install --path . --force --bin opengate-tts

install: opengate
	cargo install --path . --force
	cargo install --path . --force --bin opengate
	cargo install --path . --force --bin opengate-tts

fmt:
	cargo fmt

lint: fmt
	cargo clippy -- -D warnings

test:
	cargo test --verbose

clean:
	cargo clean

short: opengate
	test -f "./test_short.wav" && rm ./test_short.wav || true
	$(OPENGATE) ./beats/test_short.yaml --out ./test_short.wav
	aplay ./test_short.wav

short-flac: opengate-flac
	test -f "./test_short.flac" && rm ./test_short.flac || true
	$(OPENGATE) ./beats/test_short.yaml --out ./test_short.flac
	# Need sudo apt install ffmpeg for ffplay
	ffplay -autoexit -nodisp ./test_short.flac
