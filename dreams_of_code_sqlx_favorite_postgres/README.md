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

There are 4 functions available to us for fetching data:
- fetch_one
- fetch_optional
- fetch_all
- fetch


#### fetch_one

Add the following function:

`fetch_one` is used for pulling a single row out of the database.

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


#### fetch_optional

Much like `fetch`, but returning an `Option` (eg. returning `Option.None` if the query fails).

Modify your fetch_one function as follows:

```Rust
async fn fetch_one(isbn: &str, pool: &sqlx::PgPool) -> Result<Option<Book>, Box<dyn Error>> {
    let query = "SELECT title, author, isbn FROM book WHERE isbn = $1";

    let maybe_row = sqlx::query(query)
        .bind(isbn)
        .fetch_optional(pool)
        .await?;

    let book = maybe_row.map(|row| {
        Book {
            title: row.get("title"),
            author: row.get("author"),
            isbn: row.get("isbn"),
        }
    });

    Ok(book)
}
```

When running the code, if the record was found, you should see this output:

```Text
Some(
    Book {
        title: "Salem's lot",
        author: "Stephen Edvin King",
        isbn: "978-0-385-00751-1",
    },
)
```

If the record was not found you should see this:

```Text
None
```

#### fetch_all

Fetches all records matching the query and returns them as a vector that can be iterated.

Add another function fetch_all:

```Rust
async fn fetch_all(pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let query = "SELECT title, author, isbn FROM book";

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;

    let books = rows.iter().map(|row| {
        Book {
            title: row.get("title"),
            author: row.get("author"),
            isbn: row.get("isbn"),
        }
    }).collect();

    Ok(books)
}
```

Add this function to create another book instance:

```Rust
fn book_1() -> Book {
    Book { 
        title: "Rust Programming".to_string(),
        author: "Steve Klabnik".to_string(),
        isbn: "1234567890".to_string(),    
    }
}
```

Then; modify your program to create one more book and then fetch all:

```Rust
    create(&book_1(), &pool).await?;

    let books = fetch_all(&pool).await?;
```

When running the program you should see the following output:

```Text
[
    Book {
        title: "Salem's lot",
        author: "Stephen Edvin King",
        isbn: "978-0-385-00751-1",
    },
    Book {
        title: "Rust Programming",
        author: "Steve Klabnik",
        isbn: "1234567890",
    },
]
```


#### fetch

Fetches all matching documents just like `fetch_all`, but as a stream-like type. This is a better solution when working with a larger dataset.

Create a new function `fetch` with the following content:

```Rust
async fn fetch(pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = "SELECT title, author, isbn FROM book";

    let query = sqlx::query(q);

    let mut rows = query.fetch(pool);
        
    let mut books = vec![];

    while let Some(row) = rows.try_next().await? {
        books.push(
            Book {
                title: row.get("title"),
                author: row.get("author"),
                isbn: row.get("isbn"),
            }
        );
    }

    Ok(books)
}
```

Modify your program to not create another book (since this will create an error trying to insert something that's already there),
and call the new function instead:

```Rust
//create(&book_1(), &pool).await?;

let books = fetch(&pool).await?;
```

Also you need to modify your `cargo.toml` file to include the `futures` trait. 

Add it either by running the command:

```sh
cargo add futures
```

Or by editing `Cargo.toml` adding:

```toml
futures = "0.3.30"
```

to the `[dependencies]` section.


Run your program. As before you should see:

```Text
[
    Book {
        title: "Salem's lot",
        author: "Stephen Edvin King",
        isbn: "978-0-385-00751-1",
    },
    Book {
        title: "Rust Programming",
        author: "Steve Klabnik",
        isbn: "1234567890",
    },
]
```


#### query_as

SQLx can also help you converting row data into concrete types using `query_as`. For this to work the struct needs to derive the `FrownRow` on our Book struct.

```Rust
#[derive(Debug, FromRow)]
struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String
}
```

We then have to modify our function to use `query_as` instead of `query` and reference the struct (Book) we want to turn the data into.

```Rust
async fn fetch(pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = "SELECT title, author, isbn FROM book";

    let query = sqlx::query_as::<_, Book>(q);

    let books = query.fetch_all(pool).await?;
        
    Ok(books)
}
```



### Transactions

SQLx supports transactions, as shown in example below where we insert an author as part of inserting a book.

```Rust
async fn insert_book(
    book: Book, 
    conn: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    let mut txn = conn.begin().await?;

    let author_q = r"
    INSERT INTO author (name) VALUES ($1) RETURNING id
    ";

    let book_q = r"
    INSERT INTO book (title, author_id, isbn)
    VALUES ($1, $2, $3)
    ";

    let author_id: (i64,) = sqlx::query_as(author_q)
        .bind(&book.author)
        .fetch_one(&mut *txn)
        .await?;

    sqlx::query(book_q)
        .bind(&book.title)
        .bind(&author_id.0)
        .bind(&book.isbn)
        .execute(&mut *txn)
        .await?;

    // Commit transaction
    txn.commit().await?;

    // Or rollback if needed
    //txn.rollback().await?;

    Ok(())
}
```

Since we're now referencing two tables (`book` and `authors`) we have to migrate our database to a new revision.

Create file `0003_migrate_author_data.sql` in the `migrations` folder with the following content:

```SQL
ALTER TABLE book
ADD COLUMN author_id INT,
ADD CONSTRAINT fk_author
FOREIGN KEY (author_id) REFERENCES authors(id);

-- Insert unique authors into authors table
INSERT INTO authors (name)
SELECT DISTINCT author
FROM book;

-- Update the book table to point to the correct author_id
UPDATE book
SET author_id = a.id
FROM authors a
WHERE book.author = a.name;

-- Drop the old author column from book
ALTER TABLE book
DROP COLUMN IF EXISTS author;
```

Also create file `0003_migrate_author_data.down.sql` for migrating back. 

Note that files for reverting to previous version of database uses the same name but with the `down` "middle name".

The command for triggering a SQLx migration revert is:

```sh
sqlx migrate revert
```

Add the following content to this file:

```SQL
-- Re-add the author column to the book table
ALTER TABLE book
ADD COLUMN IF NOT EXISTS author TEXT;

-- Populate the author column with the concatenated names from authors
UPDATE book
SET author = a.name
FROM authors a
WHERE book.author_id = a.id;

-- Drop constraint on author
ALTER TABLE book
DROP CONSTRAINT fk_author;

-- Delete the inserted authors if they were created solely for this migration
DELETE FROM authors
WHERE id IN (
    SELECT DISTINCT author_id FROM book
);

-- Drop the foreign key constraint and the author_id column
ALTER TABLE book
DROP COLUMN author_id;
```

Modify your `fetch` function like this:

```Rust
async fn fetch(pool: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = r"
    SELECT 
        book.title as title,
        book.isbn as isbn,
        authors.name as author
    FROM 
        book
    JOIN 
        authors ON book.author_id = authors.id;
    ";

    let query = sqlx::query_as::<_, Book>(q);

    let books = query.fetch_all(pool).await?;
        
    Ok(books)
}
```


Modify your `main.rs` to call the new transactional book insert:

```Rust
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://postgres:postgres@localhost:5432/bookstore";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let book = Book {
        title: "Rust for Rustaceans: Idiomatic Programming for Experienced Developers".to_string(),
        author: "Jon Gjengset".to_string(),
        isbn: "978-1-718-50185-0".to_string(),
    };

    let books = fetch(&pool).await?;

    println!("{:#?}", books);  // Pretty-prints with indentation

    Ok(())
}
```


Run your program. After the output from the fetch should be:


```JSON
[
    Book {
        title: "Salem's lot",
        author: "Stephen King",
        isbn: "978-0-385-00751-1",
    },
    Book {
        title: "Rust Programming",
        author: "Steve Klabnik",
        isbn: "978-1-593-27828-1",
    },
    Book {
        title: "Rust for Rustaceans: Idiomatic Programming for Experienced Developers",
        author: "Jon Gjengset",
        isbn: "978-1-718-50185-0",
    },
]
```
