use std::time::Duration;

use database::{Client, Database, Equal, SchemaLeaf, SchemaNode, Value, ValueLeaf};
use tokio::join;

#[tokio::main]
async fn main() {
    let mut database = Database::new(
        SchemaNode::Product(vec![
            SchemaNode::Leaf(SchemaLeaf::String),
            SchemaNode::List(Box::new(SchemaNode::Leaf(SchemaLeaf::String))),
            SchemaNode::Leaf(SchemaLeaf::Unit),
        ]),
        Value::Product(vec![
            Value::Leaf(ValueLeaf::String("string 2".to_string())),
            Value::List(vec![
                Value::Leaf(ValueLeaf::String("string 1".to_string())),
                Value::Leaf(ValueLeaf::String("string 2".to_string())),
                Value::Leaf(ValueLeaf::String("string 1".to_string())),
                Value::Leaf(ValueLeaf::String("string 3".to_string())),
            ]),
            Value::Leaf(ValueLeaf::Unit),
        ]),
    );

    let client_future = async {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = Client::<(String, Vec<String>, ())>::connect("localhost:1234")
            .await
            .unwrap();

        dbg!(client.get_schema().await.unwrap());
        dbg!(client.query(|db| db).await.unwrap());
    };

    let (database_result, _) = join!(database.listen("localhost:1234"), client_future);

    database_result.unwrap();
}
