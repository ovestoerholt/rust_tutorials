# SQLx is my favorite PostgreSQL driver to use with Rust
Source: https://www.youtube.com/watch?v=TCERYbgvbq0

## Steps

### Set up postgres image

Check repo `docker-compose.yml`.

Start docker image using the following command:

```sh
docker compose up
```
### Create Postgres database `bookstore`

#### Shell into docker image

In another terminal window shell into the running docker image using the command:

```sh
docker exec -it <docker container id> sh
```

#### Open the provided `psql` Postgres terminal

```sh
psql postgres postgres
```

#### Create the `bookstore` database

```sql
CREATE DATABASE bookstore;
```

After test connecting to the database.

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

### Add async main

```Rust
use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello world!");
    Ok(())
}
```

### Connect to Postgres database

Modify your program so it now looks like this:

```Rust
use std::error::Error;
use sqlx::{Connection, Row};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://postgres:postgres@localhost:5432/bookstore";
    let mut conn = sqlx::postgres::PgConnection::connect(url).await?;

    let res = sqlx::query("SELECT 1 + 1 AS sum")
        .fetch_one(&mut conn)
        .await?;

    let sum: i32 = res.get("sum");
    println!("1 + 1 = {}", sum);

    Ok(())
}
```

Make sure to replace the username and password (postgres:postgres) with something matching your Postgres instance.

Run the program using the command `cargo run`.

### Add a database connection pool

Replace the line

```Rust
let mut conn = sqlx::postgres::PgConnection::connect(url).await?;
```
with
```Rust
let pool = sqlx::postgres::PgPool::connect(url).await?;
```

and make sure the line
```Rust
    .fetch_one(&mut conn)
```
references the pool instead.
```Rust
    .fetch_one(&pool)
```

You are now using a connection pool for accessing Postgres.


### Add an initial migration

SQLx supports migrations using the `migrate` macro. The migrations are by default stored in the `migrations` directory in the root of your project.


#### Create the `migrations` folder

With a terminal opened in the project root, use the following command:

```sh
mkdir migrations
```

#### Create the first migration

Inside the `migrations`folder create a file called `0001_books_table.sql`

```sh
cd migrations

touch 0001_books_table.sql
```

Note that migration files has to be in the format

```text
<version>_<description>.sql
```

Where `<version>` is a sequential number which tells SQLx in which order to run the migrations.


Edit the newly created file by adding the following SQL statement:

```sql
CREATE TABLE book (
    title varchar not null,
    author varchar not null,
    isbn varchar not null
);

CREATE UNIQUE INDEX book_isbn_idx on book(isbn);
```

#### Add running migrations to your program

Add the following line of code below the line where you declare your connection pool.

```Rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

#### Observe database changes

After look at the database content using the following command from the shell of your Docker image:

```sh
psql bookstore postgres
```

Then, at the bookstore prompt:
```sh
bookstore=# \dt
```

You should see the following:

```sh
              List of relations
 Schema |       Name       | Type  |  Owner   
--------+------------------+-------+----------
 public | _sqlx_migrations | table | postgres
 public | book             | table | postgres
(2 rows)

bookstore=# 
```


### Install `sqlx-cli`

The SQLx CLI tool helps you operate database in context of your project.

To install:

```sh
cargo install sqlx-cli
```

After the command is available using the `sqlx` command from a terminal.

#### Add buildscript

```sh
sqlx migrate build-script
```

This command adds a file `build.rs` to your project folder. 

The build-script is for running migrations when you have only modified SQL files and not Rust code.


### Add data

Create a struct with the same data as we have in our `book` table:

```Rust
struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String
}
```

Also add a function to insert a book into the `book` table:

```Rust
async fn create(book: &Book, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "INSERT INTO book (title, author, isbn) VALUES ($1, $2, $3)";
    
    sqlx::query(query)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .execute(pool)
        .await?;

    Ok(())
}
```

At last add the following lines of code below the statement running migrations and before returning `OK(())` (remove the lines related to sum calculations)

```Rust
    let book = Book {
        title: "Salem's lot".to_string(),
        author: "Stephen King".to_string(),
        isbn: "978-0-385-00751-1".to_string(),
    };

    create(&book, &pool).await?;
```

Run the program. 

After running observe the changes in the `psql` terminal session using the following command:

```sh
bookstore=# SELECT * FROM book;
```

You should see the following:

```text
    title    |    author    |       isbn        
-------------+--------------+-------------------
 Salem's lot | Stephen King | 978-0-385-00751-1
(1 row)
```


### Updating data

Add a function to update the book entry:

```Rust
async fn update(book: &Book, isbn: &str, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE book SET title = $1, author = $2 WHERE isbn = $3";
    
    sqlx::query(query)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&isbn)
        .execute(pool)
        .await?;

    Ok(())
}
```

Then modify your program code to update instead of adding:

```Rust
    //let book = Book {
    //    title: "Salem's lot".to_string(),
    //    author: "Stephen King".to_string(),
    //    isbn: "978-0-385-00751-1".to_string(),
    //};
    //
    //create(&book, &pool).await?;

    let updated_book = Book {
        title: "Salem's lot".to_string(),
        author: "Stephen Edvin King".to_string(),
        isbn: "978-0-385-00751-1".to_string(),
    };

    update(&updated_book, &updated_book.isbn, &pool).await?;
```

Check the results using `psql` terminal.


### Fetching data

#### fetch_one

Add the following function:

```Rust
async fn fetch_one(isbn: &str, pool: &sqlx::PgPool) -> Result<Book, Box<dyn Error>> {
    //let q = "SELECT (title, author, isbn) FROM book WHERE isbn = '978-0-385-00751-1'";
    let query = "SELECT title, author, isbn FROM book WHERE isbn = $1";

    let row = sqlx::query(query)
        .bind(isbn)
        .fetch_one(pool)
        .await?;

    let book = Book {
        title: row.try_get("title")?,
        author: row.try_get("author")?,
        isbn: row.try_get("isbn")?,
    };

    Ok(book)
}
```

Then comment out the part of your program updating the Stephen King record:

```Rust
    //let updated_book = Book {
    //    title: "Salem's lot".to_string(),
    //    author: "Stephen Edvin King".to_string(),
    //    isbn: "978-0-385-00751-1".to_string(),
    //};
    //
    //update(&updated_book, &updated_book.isbn, &pool).await?;

    let book = fetch_one("978-0-385-00751-1", &pool).await?;

    println!("{:#?}", book);  // Pretty-prints with indentation
```

Also; add the `Debug` trait to the `Book` struct to enable printing the struct content.

```Rust
#[derive(Debug)]
struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String
}
```

At last; make sure you import the `Row` trait:

```Rust
use sqlx::Row;
```

Run your program. You should see the following output:

```text
Book {
    title: "Salem's lot",
    author: "Stephen Edvin King",
    isbn: "978-0-385-00751-1",
}
```