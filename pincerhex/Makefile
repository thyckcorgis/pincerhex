EXE=pincerhex

all:
	cargo build --release
	rm -f ./$(EXE)
	mv -f ./target/release/$(EXE) ./$(EXE)
