SOURCE = opengate
RDIR = target/release/
OPENGATE = $(RDIR)opengate

install-ubuntu-deps:
	sudo apt install libflac-dev

build:
	cargo build --release

lint:
	cargo fmt
	cargo clippy -- -D warnings

test:
	cargo test --verbose

clean:
	cargo clean

play-short: build
	test -f "./test_short.wav" && rm ./test_short.wav || true
	$(OPENGATE) ./beats/test_short.yaml --out ./test_short.wav
	aplay ./test_short.wav

play-short-flac: build
	test -f "./test_short.flac" && rm ./test_short.flac || true
	$(OPENGATE) ./beats/test_short.yaml --out ./test_short.flac
	# Need sudo apt install ffmpeg for ffplay
	ffplay -autoexit -nodisp ./test_short.flac
