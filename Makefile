# Load .env for targets that need it (run in shell that sources .env)
export

AGENT_IMAGE ?= tompa-agent

.PHONY: run-backend run-agent build-agent run-frontend migrate backend-check frontend-check api-contract-update help

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "  run-backend   - Start backend server (requires .env)"
	@echo "  build-agent   - Build agent Docker image"
	@echo "  run-agent     - Build and run agent in Docker (requires .env)"
	@echo "                  Note: set AGENT_SERVER_URL=ws://host.docker.internal:3000 in .env"
	@echo "  run-frontend  - Start frontend dev server"
	@echo "  migrate       - Run database migrations"
	@echo "  backend-check - Format, clippy, test backend"
	@echo "  frontend-check - Typecheck, lint, test frontend"
	@echo "  api-contract-update - Regenerate OpenAPI spec and TypeScript API client"

run-backend:
	@set -a && [ -f .env ] && . ./.env && set +a && cd backend && cargo run --bin server

build-agent:
	docker buildx inspect tompa-builder > /dev/null 2>&1 || docker buildx create --name tompa-builder --driver docker-container
	docker buildx build --builder tompa-builder --load -t $(AGENT_IMAGE) -f backend/agent/Dockerfile backend

run-agent: build-agent
	docker run --rm -it \
		--env-file .env \
		--add-host=host.docker.internal:host-gateway \
		-p 3001:3001 \
		-v $(CURDIR)/agent-claude:/root/.claude \
		$(AGENT_IMAGE)

run-frontend:
	cd frontend && bun run dev

migrate:
	@set -a && [ -f .env ] && . ./.env && set +a && cd backend && sqlx migrate run --source server/migrations

backend-check:
	cd backend && cargo fmt --check
	cd backend && cargo clippy --all-targets -- -D warnings
	cd backend && cargo test

frontend-check:
	cd frontend && bun install
	cd frontend && bun run typecheck
	cd frontend && bun run lint
	cd frontend && bun run test

api-contract-update:
	cd frontend && SQLX_OFFLINE=true bun run generate-api
