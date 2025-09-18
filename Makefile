SOURCE = opengate
RDIR = target/release/
OPENGATE = $(RDIR)opengate

build:
	cargo build --release

lint:
	cargo fmt
	cargo clippy -- -D warnings

test:
	cargo test --verbose

play-short: build
	test -f "./test_short.wav" && rm ./test_short.wav || true
	$(OPENGATE) ./beats/test_short.yaml
	aplay ./test_short.wav
