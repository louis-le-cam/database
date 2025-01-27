use std::time::Duration;

use database::{Client, Database, Filter, Schema, SchemaNode, Set, StringEqual as _, Value};
use tokio::join;

#[derive(Schema, Debug)]
struct User {
    name: String,
    location: Location,
    shape: Option<Shape>,
}

#[derive(Schema, Debug)]
struct Location(f32, f32);

#[derive(Schema, Debug)]
enum Shape {
    Rectangle {
        width: f32,
        height: f32,
    },
    Triangle {
        a: (f32, f32),
        b: (f32, f32),
        c: (f32, f32),
    },
    Circle(f32),
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
                    location: Location(
                        8294829452839424242.24982438293839242,
                        -1151271151278511612757575.57121157611512611612575,
                    ),
                    shape: None,
                },
                User {
                    name: "user 2".to_string(),
                    location: Location(
                        -22722978511612757575.57121157611512611612575,
                        44845451011844945108108108.81045448109448459449458108,
                    ),
                    shape: Some(Shape::Rectangle {
                        width: 32.0,
                        height: 16.8,
                    }),
                },
            ])
            .await
            .unwrap();

        dbg!(client.query(|users| users).await.unwrap());
        dbg!(client
            .query(|users| users
                .clone()
                .set(users.filter(|user| { user.name.equal("user 1") })))
            .await
            .unwrap());
        dbg!(client.query(|users| users).await.unwrap());
    };

    let (database_result, ()) = join!(database.listen("localhost:1234"), client_future);

    database_result.unwrap();
}
