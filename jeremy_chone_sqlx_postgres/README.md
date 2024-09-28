# Rust with SQLx and Postgres tutorial

## Steps

### Set up postgres image

#### Setup and start docker image

Check repo `docker-compose.yml`.

Start docker image using the following command:

```sh
docker compose up
```

#### Shell into docker image

In another terminal window shell into the running docker image using the command:

```sh
docker exec -it <docker container id> sh
```

#### Alter Postgres logging

To see the outcome of our Postgres operations we need to change how much Postgres logs.

Open a Postgres terminal:
```sh
psql postgres postgres
```

Where the first `postgres`is the database and the second `postgres`is the username (specified in the docker-compose.yml)

Then enter the following SQL statement:
```sql
ALTER DATABASE postgres SET log_statement = 'all'
```

### Add dependencies

Tokio:
```sh
cargo add tokio --features full
```

SQLx:
```sh
cargo add sqlx --features runtime-tokio-rustls
cargo add sqlx --features postgres 
```

