use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "world".to_string()
    }
}
