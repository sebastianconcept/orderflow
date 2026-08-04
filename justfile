# orderflow — docker compose (dev / prod) and Kamal deploy helpers

set dotenv-load

compose_dev := "docker compose -f docker-compose.dev.yml"
compose_prod := "docker compose -f docker-compose.prod.yml"
kamal_config := "config/deploy.yml"

default:
    @just --list

# --- Development (docker-compose.dev.yml) ---

up:
    {{compose_dev}} up -d --build

down:
    {{compose_dev}} down --remove-orphans

start:
    {{compose_dev}} start

stop:
    {{compose_dev}} stop

logs service="":
    {{compose_dev}} logs -f {{service}}

ps:
    {{compose_dev}} ps

build:
    {{compose_dev}} build

# --- Production-like compose (docker-compose.prod.yml) ---

prod-up:
    {{compose_prod}} up -d --build

prod-down:
    {{compose_prod}} down --remove-orphans

prod-start:
    {{compose_prod}} start

prod-stop:
    {{compose_prod}} stop

prod-logs service="":
    {{compose_prod}} logs -f {{service}}

prod-ps:
    {{compose_prod}} ps

prod-build:
    {{compose_prod}} build

# --- Image build (shared by compose and Kamal) ---

docker-build tag="orderflow:local":
    docker build -t {{tag}} -f Dockerfile .

# --- Kamal (remote deploy) ---

kamal-setup:
    kamal setup -c {{kamal_config}}

kamal-deploy:
    kamal deploy -c {{kamal_config}}

kamal-redeploy:
    kamal redeploy -c {{kamal_config}}

kamal-logs role="web":
    kamal app logs -c {{kamal_config}} --roles {{role}}

kamal-details:
    kamal app details -c {{kamal_config}}

# --- Rust (local, outside docker) ---

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
