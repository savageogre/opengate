SOURCE = opengate
RDIR = target/release/
OPENGATE = $(RDIR)opengate

all: lint opengate

install-ubuntu-flac-deps:
	sudo apt update
	sudo apt install libflac-dev -y

install-ubuntu-tts-deps:
	sudo apt update
	# libssl error
	sudo apt install pkg-config libssl-dev -y
	# stddef.h error
	sudo apt install build-essential clang libclang-dev -y
	# cmake error
	sudo apt install cmake -y
	# compiling piper-rs fork
	sudo apt install libasound2-dev libespeak-ng-dev -y

opengate-tts:
	cargo build --release --bin opengate-tts

opengate-flac:
	cargo build --release --features flac

opengate:
	cargo build --release

install-tts: opengate-tts
	cargo install --path . --force --bin opengate-tts

install-flac: opengate-flac
	cargo install --path . --force

install: opengate
	cargo install --path . --force

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

short-flac: opengate
	test -f "./test_short.flac" && rm ./test_short.flac || true
	$(OPENGATE) ./beats/test_short.yaml --out ./test_short.flac
	# Need sudo apt install ffmpeg for ffplay
	ffplay -autoexit -nodisp ./test_short.flac
