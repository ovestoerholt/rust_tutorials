use std::error::Error;
use sqlx::{prelude::FromRow, Row};
use tokio;

#[derive(Debug, FromRow)]
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



fn book_1() -> Book {
    Book { 
        title: "Rust Programming".to_string(),
        author: "Steve Klabnik".to_string(),
        isbn: "1234567890".to_string(),    
    }
}


async fn insert_book(
    book: Book, 
    conn: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    let mut txn = conn.begin().await?;

    let author_q = r"
    INSERT INTO authors (name) VALUES ($1) RETURNING id
    ";

    let book_q = r"
    INSERT INTO book (title, author_id, isbn)
    VALUES ($1, $2, $3)
    ";

    let author_id: (i32,) = sqlx::query_as(author_q)
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

    //create(&book_1(), &pool).await?;

    let book = Book {
        title: "Rust for Rustaceans: Idiomatic Programming for Experienced Developers".to_string(),
        author: "Jon Gjengset".to_string(),
        isbn: "978-1-718-50185-0".to_string(),
    };

    //insert_book(book, &pool).await?;

    let books = fetch(&pool).await?;

    println!("{:#?}", books);  // Pretty-prints with indentation

    Ok(())
}
