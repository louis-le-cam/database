use std::time::Duration;

use database::{Client, Database, SchemaNode, Value};
use tokio::join;

#[tokio::main]
async fn main() {
    let mut database = Database::new(
        SchemaNode::Product(vec![
            SchemaNode::String,
            SchemaNode::List(Box::new(SchemaNode::String)),
            SchemaNode::Sum(vec![SchemaNode::Unit, SchemaNode::Uint32]),
        ]),
        Value::Product(vec![
            Value::String("string 2".to_string()),
            Value::List(vec![
                Value::String("string 1".to_string()),
                Value::String("string 2".to_string()),
                Value::String("string 1".to_string()),
                Value::String("string 3".to_string()),
            ]),
            Value::Sum(1, Box::new(Value::Uint32(8349342))),
        ]),
    );

    let client_future = async {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = Client::<(String, Vec<String>, Option<u32>)>::connect("localhost:1234")
            .await
            .unwrap();

        dbg!(client.get_schema().await.unwrap());
        dbg!(client.query(|db| db).await.unwrap());
    };

    let (database_result, ()) = join!(database.listen("localhost:1234"), client_future);

    database_result.unwrap();
}
