use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
    time::Duration,
};

use database::{
    make_keys, BoolOperators, Client, Database, MapVec, OptionOperators, Schema, SchemaNode,
    SlotMap, StringEqual, Value, VecGet as _,
};
use tokio::join;

make_keys! {
    #[derive(Debug)]
    struct UserKey;
}

#[derive(Schema, Debug)]
struct Db {
    test: Vec<String>,
    non_zero: NonZeroU32,
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
                test: vec![
                    "string 1".to_string(),
                    "string 2".to_string(),
                    "string 3".to_string(),
                ],
                non_zero: NonZeroU32::new(3).unwrap(),
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

        dbg!(client.query(|db| db).await?);
        dbg!(
            client
                .query(|db| (
                    1,
                    db.clone(),
                    Db {
                        test: Vec::new(),
                        non_zero: NonZeroU32::new(8).unwrap(),
                        users: SlotMap::new()
                    }
                ))
                .await?
        );

        dbg!(
            client
                .query(|db| db.test.map(|value| (1, value, 2)))
                .await?
        );

        for i in 0u32..4 {
            dbg!(client
                .query(|db| db
                    .test
                    .get(i)
                    .unwrap_or("default")
                    .equal("string 2")
                    .if_else(32, 89))
                .await
                .unwrap());
        }

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

    database_result.unwrap();
    client_result.unwrap();
}
