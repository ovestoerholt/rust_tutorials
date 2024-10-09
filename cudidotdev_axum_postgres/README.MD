# Build a CRUD REST API with Rust Axum | Tutorial
Source: https://www.youtube.com/watch?v=NJsTgmayHZY


## Setup your environment

### Configure Postgres environment

#### Add docker environment

Add a `docker-compose.yml` file to the root of your project with the following content:

```yml
services:
  db:
    image: postgres:17.0-bookworm
    restart: always
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
    ports:
      - "5432:5432"
    volumes:
      - db:/var/lib/postgresql/data
volumes:
  db:
    driver: local
```

Then start the docker environment (the database):

```sh
docker compose up
```

#### Create Postgres database and database owner (role)

Connect to the database server either using the docker console `psql`or by connecting from remote using fx. [Azure Data Studio](https://azure.microsoft.com/en-us/products/data-studio).

Run the following queries to create user and database:

```SQL
CREATE ROLE axum_postgres WITH LOGIN PASSWORD 'axum_postgres';
```

```SQL
CREATE DATABASE axum_postgres WITH OWNER = 'axum_postgres';
```

### Environment variables

### Add the `dotenv` crate to your project

Add this to the `[dependencies]` section of `cargo.toml`

```toml
[dependencies]
dotenv = "0.15.0"
```

or; add by command

```sh
cargo add dotenv
```

### Modify your `main.rs` file

```rust
use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    println!("DATABASE_URL: {}", database_url);
}
```

### Add a `.env` file to the root of your project

```Text
DATABASE_URL=postgres://axum_postgres:axum_postgres@localhost/axum_postgres
```

### Run your program

```sh
cargo run
```

You should see the following output:

```Text
DATABASE_URL: postgres://axum_postgres:axum_postgres@localhost/axum_postgres
```

