use surrealdb::{error::Db, Error};

use crate::{domain::models::todo::Todo, infrastructure::db_context::surreal_context::DB};

pub struct TodoRepository {
    table: String,
}

impl TodoRepository {
    pub fn new() -> Self {
        TodoRepository {
            table: String::from("todo"),
        }
    }

    pub async fn get_all(&self) -> Result<Vec<Todo>, Error> {
        let records = DB.select(&self.table).await?;
        Ok(records)
    }

    pub async fn get_by_id(&self, id: String) -> Result<Todo, Error> {
        if let Some(todo) = DB.select((&self.table, id)).await? {
            Ok(todo)
        } else {
            Err(Error::Db(Db::Thrown("Todo not found".into())))
        }
    }

    pub async fn get_by_title(&self, title: String) -> Result<Todo, Error> {
        if let Some(record) = DB
            .query("SELECT * FROM todo WHERE title = $title")
            .bind(("title", title.clone()))
            .await?
            .take(0)?
        {
            return Ok(record);
        }

        let error = Error::Db(Db::Thrown(format!("Todo with title {} not found", title)));
        Err(error)
    }

    pub async fn create_todo(&self, content: Todo) -> Result<Option<Todo>, Error> {
        let record = DB.create(&self.table).content(content).await?;
        Ok(record)
    }

    pub async fn update_todo(&self, content: Todo) -> Result<Todo, Error> {
        let content_id = match content._id.clone() {
            Some(content_id) => content_id,
            None => todo!(),
        };

        let record = DB
            .update((&self.table, content_id))
            .content(content)
            .await?
            .unwrap();

        Ok(record)
    }

    pub async fn delete_todo(&self, id: String) -> Result<Todo, Error> {
        let result = DB.delete((&self.table, id)).await?.unwrap();
        Ok(result)
    }
}
