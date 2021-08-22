use crate::calibrate::{
    client::HttpClient,
    Result,
};
use serde::{Serialize, Deserialize};
// use warp::{reject, reply::json, Reply};

#[derive(Deserialize)]
pub struct Todo {
    pub id: i32,
    pub name: String,
    pub checked: bool,
}

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: i32,
    pub name: String,
    pub checked: bool,
}

impl TodoResponse {
    pub fn of(todo: Todo) -> TodoResponse {
        TodoResponse {
            id: todo.id,
            name: todo.name,
            checked: todo.checked,
        }
    }
}

// pub async fn list_todos_handler() -> Result<impl Reply> {
//     let todos = vec![
//         Todo { id: 1, name: "Hello".to_string(), checked: false },
//         Todo { id: 2, name: "World".to_string(), checked: true }];
//     Ok(json::<Vec<_>>(
//         &todos.into_iter().map(|t| TodoResponse::of(t)).collect(),
//     ))
// }

// pub async fn create_todo(
//     http_client: impl HttpClient,
// ) -> Result<impl Reply> {
//     let todos = vec!["Hello".to_string(), "World".to_string()];
//     let cat_fact = http_client
//         .get_cat_fact()
//         .await
//         .map_err(|e| reject::custom(e))?;
//     Ok(json(&todos))
// }
