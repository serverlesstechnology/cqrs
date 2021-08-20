
all:
	make clean
	make build
	make doc

clean:
	rm -rf target
	rm -rf Cargo.lock
	rm -rf test.db*

build:
	cargo build

up:
	docker-compose up -d
	rm -rf test.db*

down:
	docker-compose down

test:
	cargo test

doc:
	cargo doc --lib --no-deps

deploy:
	cargo publish --token ${CRATES_IO_TOKEN}

check:
	cargo publish --dry-run --allow-dirty
