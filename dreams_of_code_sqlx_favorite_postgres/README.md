# SQLx is my favorite PostgreSQL driver to use with Rust
Source: https://www.youtube.com/watch?v=TCERYbgvbq0

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

Then run `cargo check` to make sure everything installs correctly.

