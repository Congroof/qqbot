export DATA_DIR=./data/bot

dev:
	cargo run --bin napcat-bot

build:
	docker-compose build

up:
	NAPCAT_UID=$(shell id -u) NAPCAT_GID=$(shell id -g) docker-compose up -d

deploy:
	docker-compose build napcat-bot
	NAPCAT_UID=$(shell id -u) NAPCAT_GID=$(shell id -g) docker-compose up -d napcat-bot

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
