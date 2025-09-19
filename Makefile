SOURCE = opengate
RDIR = target/release/
OPENGATE = $(RDIR)opengate

all: lint opengate

install-ubuntu-deps:
	sudo apt install libflac-dev

opengate:
	cargo build --release

install: opengate
	cargo install --path .

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
