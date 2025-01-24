use std::{borrow::Cow, marker::PhantomData, pin::Pin};

use database::{
    pipeline::{Condition, Stage, ValueOrPath},
    request::Request,
    schema::SchemaNode,
    value::Value,
};
use tokio::io::AsyncRead;

struct TcpRead(tokio::net::tcp::OwnedReadHalf);

impl AsyncRead for TcpRead {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let poll = AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf);

        // for byte in buf.filled() {
        //     print!("{byte:02x} ");
        // }

        poll
    }
}

#[tokio::main]
async fn main() {
    let tcp = tokio::net::TcpStream::connect("localhost:8001")
        .await
        .unwrap();

    let (read, mut write) = tcp.into_split();

    let mut read = TcpRead(read);

    Request::GetSchema.write(&mut write).await.unwrap();
    let schema = SchemaNode::read(&mut read).await.unwrap();

    let stages = [
        Stage::Get {
            path: (&[0, 1, 0, 1]).into(),
        },
        Stage::Set {
            destination: (&[0, 1, 0, 0]).into(),
            value: ValueOrPath::Path((&[]).into()),
        },
        Stage::Get { path: (&[]).into() },
        Stage::Filter {
            path: (&[]).into(),
            condition: Condition::Equal {
                schema: SchemaNode::String,
                lhs: ValueOrPath::Path((&[0]).into()),
                rhs: ValueOrPath::Value(Value::String("test name".to_string())),
            },
        },
    ];

    Request::Pipeline(Cow::Borrowed(&stages))
        .write(&mut write)
        .await
        .unwrap();

    let mut response_schema = Cow::Borrowed(&schema);

    for stage in &stages {
        // TODO: do not into_owned here
        response_schema = Cow::Owned(
            stage
                .transform_schema(&schema, response_schema.as_ref())
                .into_owned(),
        );
    }

    let data = Value::read(&response_schema, &mut read).await.unwrap();

    dbg!(data);
}

// trait Schema {
//     fn schema_node() -> SchemaNode;
// }

// struct Database<S: Schema> {
//     _marker: PhantomData<S>,
// }

// impl<S: Schema> Database<S> {
//     pub fn connect() -> Self {
//         todo!()
//     }
// }

// struct User {
//     name: String,
//     location: Option<(u64, u64)>,
// }

// // impl Schema for User {
// //     const SCHEMA_NODE: SchemaNode<'static> = SchemaNode::Product();
// // }

// async fn t() {
//     let database = Database::<Vec<User>>::connect();

//     // Change the name of every user
//     database
//         .pipeline()
//         .set(|schema| schema.all().name = "test")
//         .await;

//     // Get user by name
//     database
//         .pipeline()
//         .filter(
//             // On which array should we apply the filter
//             |schema| schema,
//             // Predicate
//             |schema| schema.name == "the searched name",
//         )
//         // TODO: how do we manage the case where there is no result ?
//         .select(|schema| schema[0])
//         .await;

//     // Change the location of specific user
//     database
//         .pipeline()
//         .filter(|schema| schema, |schema| schema.name == "the specific user")
//         .set(|schema| schema.location = Some((923, 142)));
// }
