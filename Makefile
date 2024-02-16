.PHONY: up

up:
	docker compose -f compose.dev.yml up

up-build:
	docker compose -f compose.dev.yml up --build
