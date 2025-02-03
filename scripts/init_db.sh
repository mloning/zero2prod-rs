#!usr/bin/env bash
set -eox pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  exit 1
fi

CONTAINER_NAME="postgres-test"
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"
IMAGE="postgres"
N_CONNECTIONS=1000

if [[ -z ${SKIP_DOCKER} ]]; then
  docker run \
  --env POSTGRES_USER=${DB_USER} \
  --env POSTGRES_PASSWORD=${DB_PASSWORD} \
  --env POSTGRES_DB=${DB_NAME} \
  --publish "${DB_PORT}":5432 \
  --name "${CONTAINER_NAME}" \
  --detach \
  "${IMAGE}" \
  postgres -N ${N_CONNECTIONS}
fi

until psql --host "${DB_HOST}" --username "${DB_USER}" --port "${DB_PORT}" --dbname "postgres" --command '\q'; do
  >&2 echo "Postgres is still unavailable, waiting ..."
  sleep 1
done

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run
