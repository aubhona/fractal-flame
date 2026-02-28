.PHONY: build-backend build-frontend build-all deploy helm-install helm-upgrade helm-uninstall dev-up dev-down

# Локальная разработка: поднять Redis и MinIO
dev-up:
	docker compose up -d

dev-down:
	docker compose down

# Сборка образов
build-backend:
	docker build -t fractal-flame-backend:latest -f crates/fractal-flame-backend/Dockerfile .

build-frontend:
	docker build -t fractal-flame-frontend:latest -f crates/fractal-flame-frontend/Dockerfile .

build-all: build-backend build-frontend

# Helm: установка/обновление (release name = fractal-flame)
deploy: build-all
	helm upgrade --install fractal-flame ./deploy/fractal-flame

helm-install:
	helm upgrade --install fractal-flame ./deploy/fractal-flame

helm-upgrade:
	helm upgrade fractal-flame ./deploy/fractal-flame

helm-uninstall:
	helm uninstall fractal-flame
