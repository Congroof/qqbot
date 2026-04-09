dev:
	cargo run --bin napcat-bot

build:
	docker-compose build

up:
	NAPCAT_UID=$(shell id -u) NAPCAT_GID=$(shell id -g) docker-compose up -d

down:
	docker-compose down

restart:
	docker-compose restart

logs:
	docker-compose logs -f

logs-bot:
	docker-compose logs -f napcat-bot

clean:
	docker-compose down -v
