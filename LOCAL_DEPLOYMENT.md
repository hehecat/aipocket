# Isolated local deployment

This stack is intended for local acceptance and smoke testing. It uses mock-only credentials, project-scoped named volumes, loopback-only configurable ports, and no host `/data` bind mount.

```bash
COMPOSE_PROJECT_NAME=aipocket-t3 \
AIPOCKET_WEB_PORT=13080 \
AIPOCKET_API_PORT=18000 \
docker compose --env-file compose.local.env -f compose.local.yml up -d --build --wait

curl --fail http://127.0.0.1:18000/api/health
curl --fail http://127.0.0.1:13080/api/health
```

Open `http://127.0.0.1:13080` and sign in with the mock password from `compose.local.env`. Do not add real provider credentials to this stack.

Clean up the isolated containers, networks, and named volumes:

```bash
COMPOSE_PROJECT_NAME=aipocket-t3 \
docker compose --env-file compose.local.env -f compose.local.yml down --volumes --remove-orphans
```
