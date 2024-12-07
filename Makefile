.PHONY: test

DB_NAME := rusteam
DB_USER := admin
DB_PASSWORD := admin

default: build/release

bootstrap:
	@./scripts/bootstrap.sh

build:
	cargo build

build/release:
	cargo build --release

test:
	@./scripts/test.sh

install:
	sudo cp target/release/rusteam /usr/local/bin/rusteam

run-sync: bootstrap
	cargo run -- sync

destroy:
	docker compose down --volumes

superset-up: bootstrap
	cd superset && docker compose up -d

superset-down:
	cd superset && docker compose down
