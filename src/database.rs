use std::{
    convert::Infallible,
    io,
    sync::{Arc, Mutex},
};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use crate::{io_error, ExpressionNode, SchemaNode, Value};

pub struct Database {
    schema: Arc<Mutex<SchemaNode>>,
    value: Arc<Mutex<Value>>,
}

impl Database {
    pub fn new(schema: SchemaNode, value: Value) -> Database {
        Self {
            schema: Arc::new(Mutex::new(schema)),
            value: Arc::new(Mutex::new(value)),
        }
    }

    pub async fn listen(&mut self, address: impl ToSocketAddrs) -> io::Result<Infallible> {
        let listener = TcpListener::bind(address).await?;

        println!("listening on {}", listener.local_addr().unwrap());

        loop {
            let (tcp, address) = listener.accept().await?;
            println!("({address}) connection accepted");

            if let Err(err) = self.handle_connection(tcp).await {
                println!("({address}) connection closed, error: {err}");
            }
        }
    }

    pub async fn handle_connection(&self, mut tcp: TcpStream) -> io::Result<()> {
        loop {
            match tcp.read_u8().await? {
                0 => self.schema.lock().unwrap().write(&mut tcp).await?,
                1 => {
                    let schema = SchemaNode::read(&mut tcp).await?;
                    let value = Value::read(&schema, &mut tcp).await?;

                    *self.schema.lock().unwrap() = schema;
                    *self.value.lock().unwrap() = value;
                }
                2 => {
                    ExpressionNode::read(&mut tcp)
                        .await?
                        .evaluate(vec![self.value.clone()])
                        .lock()
                        .unwrap()
                        .write(&mut tcp)
                        .await?;
                }
                _ => return Err(io_error!(InvalidData, "invalid discriminant for request")),
            }
        }
    }
}
