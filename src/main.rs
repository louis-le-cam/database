use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use database::{Client, Database, Equal, Filter, SchemaNode, Set, Value};
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
            Arc::new(Mutex::new(Value::String("string 2".to_string()))),
            Arc::new(Mutex::new(Value::List(vec![
                Arc::new(Mutex::new(Value::String("string 1".to_string()))),
                Arc::new(Mutex::new(Value::String("string 2".to_string()))),
                Arc::new(Mutex::new(Value::String("string 1".to_string()))),
                Arc::new(Mutex::new(Value::String("string 3".to_string()))),
            ]))),
            Arc::new(Mutex::new(Value::Sum(
                1,
                Arc::new(Mutex::new(Value::Uint32(8349342))),
            ))),
        ]),
    );

    let client_future = async {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = Client::<(String, Vec<String>, Option<u32>)>::connect("localhost:1234")
            .await
            .unwrap();

        dbg!(client.get_schema().await.unwrap());
        // dbg!(client.query(|db| db.0.equal(db.1.get(1))).await.unwrap());
        dbg!(client.query(|db| db).await.unwrap());
        dbg!(client
            .query(|db| Filter::filter(db.1, |string| string.equal(db.0)))
            .await
            .unwrap());
        dbg!(client.query(|db| db).await.unwrap());
        dbg!(client.query(|db| db.0.set(db.1.get(0))).await.unwrap());
        dbg!(client.query(|db| db).await.unwrap());
        dbg!(client
            .query(|db| Filter::filter(db.1, |string| string.equal(db.0)))
            .await
            .unwrap());
    };

    let (database_result, ()) = join!(database.listen("localhost:1234"), client_future);

    database_result.unwrap();
}
