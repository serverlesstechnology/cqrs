
all:
	make clean
	make build
	make doc

clean:
	rm -rf target

build:
	cargo build

up:
	docker-compose up -d

down:
	docker-compose down

test:
	cargo test

doc:
	cargo doc --lib --no-deps

deploy:
	cargo publish --token ${CRATES_IO_TOKEN}
