
all:
	make clean
	make build

clean:
	rm -rf target
	rm -rf Cargo.lock
	rm -rf *.log
	rm -rf demo.db*

build:
	cargo build

test:
	cargo test

up:
	docker-compose up -d
	rm -rf demo.db*

down:
	docker-compose down

run:
	cargo run
