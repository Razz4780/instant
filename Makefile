.PHONY: clean

all: compiler

compiler:
	cargo build --locked --release
	cp target/release/instant .

clean:
	rm -rf target instant examples/*.j examples/*.class examples/*.ll examples/*.bc examples/*result*
