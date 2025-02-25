use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use database::{Chain, Client, Database, Filter, Schema, SchemaNode, Set, StringEqual as _, Value};
use tokio::join;

#[derive(Schema, Debug)]
struct User {
    name: String,
    location: Location,
    duration: Duration,
    tags: HashSet<String>,
    shapes: HashMap<String, Shape>,
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
    let (server_stream, client_stream) = tokio::io::duplex(64);

    let client_future = async {
        let mut client = Client::<(), _>::new(client_stream)
            .await?
            .set(vec![
                User {
                    name: "user 1".to_string(),
                    location: Location(
                        8294829452839424242.24982438293839242,
                        -1151271151278511612757575.57121157611512611612575,
                    ),
                    duration: Duration::from_secs_f32(12.48),
                    tags: HashSet::from(["tag 1".to_string(), "tag 2".to_string()]),
                    shapes: HashMap::from([
                        ("shape 1".to_string(), Shape::Circle(3.0)),
                        (
                            "shape 2".to_string(),
                            Shape::Rectangle {
                                width: 23.0,
                                height: 2.8,
                            },
                        ),
                    ]),
                },
                User {
                    name: "user 2".to_string(),
                    location: Location(
                        -22722978511612757575.57121157611512611612575,
                        44845451011844945108108108.81045448109448459449458108,
                    ),
                    duration: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                    tags: HashSet::from(["tag 2".to_string(), "tag 3".to_string()]),
                    shapes: HashMap::from([
                        (
                            "shape 2".to_string(),
                            Shape::Rectangle {
                                width: 23.0,
                                height: 2.8,
                            },
                        ),
                        ("shape 3".to_string(), Shape::Circle(0.2)),
                    ]),
                },
            ])
            .await?;

        dbg!(client.query(|users| users).await?);
        dbg!(
            client
                .query(|users| users
                    .clone()
                    .set(users.clone().filter(|user| { user.name.equal("user 1") }))
                    .chain(users))
                .await?
        );

        Ok(()) as Result<(), std::io::Error>
    };

    let database = Database::new(SchemaNode::Unit, Value::Unit);
    let (database_result, client_result) = join!(database.listen(server_stream), client_future);

    database_result.unwrap();
    client_result.unwrap();
}
