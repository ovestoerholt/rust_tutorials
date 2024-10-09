use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    println!("DATABASE_URL: {}", database_url);
}