use std::{
    convert::Infallible,
    io,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, ToSocketAddrs},
};

use crate::{io_error, ExpressionNode, SchemaNode, Value};

pub struct Server {
    schema: Arc<Mutex<SchemaNode>>,
    value: Arc<Mutex<Value>>,
}

impl Server {
    pub fn new(schema: SchemaNode, value: Value) -> Self {
        Self {
            schema: Arc::new(Mutex::new(schema)),
            value: Arc::new(Mutex::new(value)),
        }
    }

    pub async fn listen_tcp(&self, address: impl ToSocketAddrs) -> io::Result<Infallible> {
        let listener = TcpListener::bind(address).await?;

        println!("listening on {}", listener.local_addr().unwrap());

        loop {
            let (tcp, address) = listener.accept().await?;
            println!("({address}) connection accepted");

            if let Err(err) = self.listen(tcp).await {
                println!("({address}) connection closed, error: {err}");
            }
        }
    }

    pub async fn listen(
        &self,
        mut stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
    ) -> io::Result<()> {
        loop {
            match stream.read_u8().await {
                Ok(0) => self.schema.lock().unwrap().write(&mut stream).await?,
                Ok(1) => {
                    let schema = SchemaNode::read(&mut stream).await?;
                    let value = Value::read(&schema, &mut stream).await?;

                    *self.schema.lock().unwrap() = schema;
                    *self.value.lock().unwrap() = value;
                }
                Ok(2) => {
                    ExpressionNode::read(&mut stream)
                        .await?
                        .evaluate(vec![self.value.clone()])
                        .lock()
                        .unwrap()
                        .write(&mut stream)
                        .await?;
                }
                Ok(_) => return Err(io_error!(InvalidData, "invalid discriminant for request")),
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break Ok(()),
                Err(err) => return Err(err),
            }
        }
    }
}
