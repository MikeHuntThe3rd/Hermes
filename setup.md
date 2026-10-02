# SQLx Macro Setup Cheat Sheet

Assumes Docker and `cargo`/Rust toolchain are already installed.

---

## 1. Setup From Scratch

### NixOS

```bash
# 1. Start Postgres container
docker run --name sqlx-dev \
  -e POSTGRES_PASSWORD=pw \
  -e POSTGRES_DB=mydb \
  -p 5432:5432 \
  -d postgres

# 2. Wait for it to be ready
docker exec sqlx-dev pg_isready -U postgres

# 3. Copy schema into the container and apply it
docker cp hermes.sql sqlx-dev:/schema.sql
docker exec sqlx-dev psql -U postgres -d mydb -f /schema.sql

# 4. Point sqlx at the DB
echo 'DATABASE_URL=postgresql://postgres:pw@localhost:5432/mydb' > .env

# 5. Install sqlx-cli (rustls avoids NixOS openssl/pkg-config linking issues)
nix-shell -p pkg-config --run "cargo install sqlx-cli --no-default-features --features rustls,postgres"

# 6. Sanity check the project compiles against the live DB
cargo check

# 7. Generate the offline query cache
cargo sqlx prepare --workspace

# 8. Verify the cache was written
ls .sqlx

# 9. Commit the cache
git add .sqlx
git commit -m "add sqlx offline query cache"

# 10. (optional) confirm offline mode works standalone
unset DATABASE_URL
SQLX_OFFLINE=true cargo check
```

> If step 5 fails looking for openssl specifically (native-tls build), use:
> `nix-shell -p openssl.dev pkg-config --run "cargo install sqlx-cli --no-default-features --features native-tls,postgres"`

### Windows (PowerShell)

```powershell
# 1. Start Postgres container
docker run --name sqlx-dev `
  -e POSTGRES_PASSWORD=pw `
  -e POSTGRES_DB=mydb `
  -p 5432:5432 `
  -d postgres

# 2. Wait for it to be ready
docker exec sqlx-dev pg_isready -U postgres

# 3. Copy schema into the container and apply it
docker cp hermes.sql sqlx-dev:/schema.sql
docker exec sqlx-dev psql -U postgres -d mydb -f /schema.sql

# 4. Point sqlx at the DB
"DATABASE_URL=postgresql://postgres:pw@localhost:5432/mydb" | Out-File -Encoding ascii .env

# 5. Install sqlx-cli
cargo install sqlx-cli --no-default-features --features rustls,postgres

# 6. Sanity check the project compiles against the live DB
cargo check

# 7. Generate the offline query cache
cargo sqlx prepare --workspace

# 8. Verify the cache was written
dir .sqlx

# 9. Commit the cache
git add .sqlx
git commit -m "add sqlx offline query cache"

# 10. (optional) confirm offline mode works standalone
Remove-Item Env:DATABASE_URL
$env:SQLX_OFFLINE = "true"
cargo check
```

---

## 2. Setup When the Container Already Exists

### NixOS / Windows (same commands)

```bash
# Restart the existing container instead of recreating it
docker start sqlx-dev

# Confirm it's ready
docker exec sqlx-dev pg_isready -U postgres

# Make sure .env / DATABASE_URL still points at it
# (only needed if you cleared the env var or deleted .env)
echo 'DATABASE_URL=postgresql://postgres:pw@localhost:5432/mydb' > .env

# Re-check and re-generate the cache after any query changes
cargo check
cargo sqlx prepare --workspace
```

PowerShell equivalent for the `.env` line:
```powershell
"DATABASE_URL=postgresql://postgres:pw@localhost:5432/mydb" | Out-File -Encoding ascii .env
```

> Re-run `cargo sqlx prepare --workspace` any time you add or edit a `query!`/`query_as!` call — the cache only updates for changed queries, but it won't update itself automatically.

---

## 3. Cleanup / Deletion

### Stop only (keep data, resume later)

```bash
docker stop sqlx-dev
# ...later...
docker start sqlx-dev
```

### Remove container (wipes data, keeps the `postgres` image cached)

```bash
docker rm -f sqlx-dev
```

### Remove container + image (full reset)

```bash
docker rm -f sqlx-dev
docker rmi postgres
```

### Nuclear option — clears ALL stopped containers, unused images, build cache (not scoped to this project)

```bash
docker system prune -a
```

### Reset data but keep the same setup (most common workflow)

```bash
docker rm -f sqlx-dev
docker run --name sqlx-dev \
  -e POSTGRES_PASSWORD=pw \
  -e POSTGRES_DB=mydb \
  -p 5432:5432 \
  -d postgres
# then redo schema load (step 3 under "Setup From Scratch")
```
