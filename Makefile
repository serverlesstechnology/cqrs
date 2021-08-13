
all:
	make clean
	make build
	make test
	make doc

clean:
	rm -rf target

build:
	cargo build

test:
	cargo test

doc:
	cargo doc --lib --no-deps

deploy:
	cargo publish --token ${CRATES_IO_TOKEN}
