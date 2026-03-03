# Load .env for targets that need it (run in shell that sources .env)
export

.PHONY: run-backend run-agent run-frontend migrate backend-check frontend-check help

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "  run-backend   - Start backend server (requires .env)"
	@echo "  run-agent     - Start agent (requires .env)"
	@echo "  run-frontend  - Start frontend dev server"
	@echo "  migrate       - Run database migrations"
	@echo "  backend-check - Format, clippy, test backend"
	@echo "  frontend-check - Typecheck, lint, test frontend"

run-backend:
	@set -a && [ -f .env ] && . ./.env && set +a && cd backend && cargo run --bin server

run-agent:
	@set -a && [ -f .env ] && . ./.env && set +a && cd backend && cargo run --bin agent

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
