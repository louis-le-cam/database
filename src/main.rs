use std::time::Duration;

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
    let mut database = Database::new(SchemaNode::Unit, Value::Unit);

    let client_future = async {
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = Client::<()>::connect("localhost:1234")
            .await
            .unwrap()
            .set(vec![
                User {
                    name: "user 1".to_string(),
                    email: None,
                },
                User {
                    name: "user 2".to_string(),
                    email: Some("user2@mail.xyz".to_string()),
                },
            ])
            .await
            .unwrap();

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
