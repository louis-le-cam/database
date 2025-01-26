use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use database::{derive_schema, Client, Database, Equal, Filter, SchemaNode, Set, Value};
use tokio::join;

derive_schema! {
    #[derive(Debug)]
    struct User {
        name: String,
        email: Option<String>,
    }
}

#[tokio::main]
async fn main() {
    let mut database = Database::new(
        SchemaNode::List(Box::new(SchemaNode::Product(vec![
            SchemaNode::String,
            SchemaNode::Sum(vec![SchemaNode::Unit, SchemaNode::String]),
        ]))),
        Value::List(vec![
            Arc::new(Mutex::new(Value::Product(vec![
                Arc::new(Mutex::new(Value::String("user 1".to_string()))),
                Arc::new(Mutex::new(Value::Sum(0, Arc::new(Mutex::new(Value::Unit))))),
            ]))),
            Arc::new(Mutex::new(Value::Product(vec![
                Arc::new(Mutex::new(Value::String("user 2".to_string()))),
                Arc::new(Mutex::new(Value::Sum(
                    1,
                    Arc::new(Mutex::new(Value::String("user2@mail.xyz".to_string()))),
                ))),
            ]))),
        ]),
    );

    let client_future = async {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = Client::<Vec<User>>::connect("localhost:1234")
            .await
            .unwrap();

        dbg!(client.get_schema().await.unwrap());
        dbg!(client.query(|users| users).await.unwrap());
        dbg!(client
            .query(|users| users
                .clone()
                .set(users.filter(|user| user.name.equal("user 1"))))
            .await
            .unwrap());
        dbg!(client.query(|users| users).await.unwrap());
    };

    let (database_result, ()) = join!(database.listen("localhost:1234"), client_future);

    database_result.unwrap();
}
