use std::{io, sync::Mutex};

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{Request, Schema, SchemaNode, Value};

crate::schema! {
    struct User {
        name: String,
        location: Option<(f32, f32)>,
    }
}

struct Database {
    schema: SchemaNode<'static>,
    value: Value<'static>,
}

pub struct Server {
    database: Mutex<Database>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            database: Mutex::new(Database {
                schema: <Vec<User> as Schema>::SCHEMA_NODE,
                value: vec![
                    User {
                        name: "user 1".to_string(),
                        location: None,
                    },
                    User {
                        name: "user 2".to_string(),
                        location: Some((-32.0, 13.5)),
                    },
                    User {
                        name: "user 3".to_string(),
                        location: Some((-28924.0, -282.5)),
                    },
                    User {
                        name: "user 4".to_string(),
                        location: None,
                    },
                ]
                .value()
                .into_owned(),
            }),
        }
    }

    pub async fn listen(&self, address: impl ToSocketAddrs) -> io::Result<()> {
        let listener = tokio::net::TcpListener::bind(address).await?;
        println!("listening on {}", listener.local_addr().unwrap());

        loop {
            let (tcp, address) = match listener.accept().await {
                Ok(ok) => ok,
                Err(err) => {
                    eprintln!("failed to accept connection, {err}");
                    continue;
                }
            };

            println!("({address}) connection accepted");

            if let Err(err) = self.handle_connection(tcp).await {
                println!("({address}) closing with error: {err}");
            }

            println!("({address}) closed");
        }
    }

    async fn handle_connection(&self, mut tcp: TcpStream) -> io::Result<()> {
        while let Some(request) = Request::read(&mut tcp).await? {
            match request {
                Request::GetSchema => {
                    self.database.lock().unwrap().schema.write(&mut tcp).await?;
                }
                Request::SetSchema(schema, value) => {
                    let mut database = self.database.lock().unwrap();
                    database.schema = schema;
                    database.value = value;
                }
                Request::Get => {
                    self.database.lock().unwrap().value.write(&mut tcp).await?;
                }
            }
        }

        Ok(())
    }
}
