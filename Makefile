DB_NAME := rusteam
DB_USER := admin
DB_PASSWORD := admin

bootstrap:
	@./scripts/bootstrap.sh

destroy:
	docker compose down --volumes

superset-up:
	cd superset && docker compose up -d

superset-down:
	cd superset && docker compose down
