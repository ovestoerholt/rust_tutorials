use std::error::Error;
use sqlx::Row;
use tokio;

#[derive(Debug)]
struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://postgres:postgres@localhost:5432/bookstore";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    //let book = Book {
    //    title: "Salem's lot".to_string(),
    //    author: "Stephen King".to_string(),
    //    isbn: "978-0-385-00751-1".to_string(),
    //};
    //
    //create(&book, &pool).await?;

    //let updated_book = Book {
    //    title: "Salem's lot".to_string(),
    //    author: "Stephen Edvin King".to_string(),
    //    isbn: "978-0-385-00751-1".to_string(),
    //};
    //
    //update(&updated_book, &updated_book.isbn, &pool).await?;

    let book = fetch_one("978-0-385-00751-1-1", &pool).await?;

    println!("{:#?}", book);  // Pretty-prints with indentation

    Ok(())
}