use std::time::Duration;

use database::{derive_schema, Client, Database, Equal, Filter, SchemaNode, Set, Value};
use tokio::join;

derive_schema! {
    #[derive(Debug)]
    struct User {
        name: String,
        email: Option<String>,
        test1: u128,
        test2: i16,
        test3: f32,
        test4: f64,
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
                    test1: 89482942842794729472894721842947297494,
                    test2: -12,
                    test3: 8294829452839424242.24982438293839242,
                    test4: 8294829452839424242.24982438293839242,
                },
                User {
                    name: "user 2".to_string(),
                    email: Some("user2@mail.xyz".to_string()),
                    test1: 91059310539538105831058391058329531058,
                    test2: 1832,
                    test3: -1151271151278511612757575.57121157611512611612575,
                    test4: -1151271151278511612757575.57121157611512611612575,
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
