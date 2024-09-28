# Rust with SQLx and Postgres tutorial

## Steps

### Set up postgres image

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

