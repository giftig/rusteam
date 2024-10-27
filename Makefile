.PHONY: test

DB_NAME := rusteam
DB_USER := admin
DB_PASSWORD := admin

bootstrap:
	@./scripts/bootstrap.sh

test:
	@./scripts/test.sh

run-sync: bootstrap
	cargo run -- sync

destroy:
	docker compose down --volumes

superset-up: bootstrap
	cd superset && docker compose up -d

superset-down:
	cd superset && docker compose down
