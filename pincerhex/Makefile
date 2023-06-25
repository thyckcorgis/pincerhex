EXE=pincerhex

all:
	cargo build --release
	rm -f ./$(EXE)
	mv -f ./target/release/$(EXE) ./$(EXE)

explore:
	cargo build --release --features explore
	rm -f ./$(EXE)_exp
	mv -f ./target/release/$(EXE) ./$(EXE)_exp

switcheroo:
	cd ../hexterminator && $(MAKE) && cp -f hexbot ../pincerhex/hexterminator
	cargo build --release --features switcheroo
	rm -f ./$(EXE)_switcheroo
	mv -f ./target/release/$(EXE) ./$(EXE)_switcheroo

alt:
	cargo build --release --features params
	rm -f ./$(EXE)_alt
	mv -f ./target/release/$(EXE) ./$(EXE)_alt
