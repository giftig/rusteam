DB_NAME := rusteam
DB_USER := admin
DB_PASSWORD := admin

bootstrap:
	@./scripts/bootstrap.sh

destroy:
	docker compose down --volumes
