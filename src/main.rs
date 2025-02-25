use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use database::{make_keys, Client, Database, Schema, SchemaNode, SlotMap, Value};
use tokio::join;

make_keys! {
    #[derive(Debug)]
    struct UserKey;
}

#[derive(Schema, Debug)]
struct Db {
    users: SlotMap<UserKey, User>,
}

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
            .set(Db {
                users: [
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
                        // duration: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                        duration: Duration::from_secs(3),
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
                ]
                .into_iter()
                .collect(),
            })
            .await?;

        dbg!(client.query(|db| db.users).await?);
        // dbg!(
        //     client
        //         .query(|db| db
        //             .users
        //             .clone()
        //             .set(
        //                 db.users
        //                     .clone()
        //                     .filter(|user| { user.name.equal("user 1") })
        //             )
        //             .chain(db.users))
        //         .await?
        // );

        Ok(()) as Result<(), std::io::Error>
    };

    let database = Database::new(SchemaNode::Unit, Value::Unit);
    let (database_result, client_result) = join!(database.listen(server_stream), client_future);

    client_result.unwrap();
    database_result.unwrap();
}
