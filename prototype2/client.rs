use std::{io, marker::PhantomData};

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{Request, Schema, SchemaNode, Value};

pub struct Client<S: Schema> {
    tcp: TcpStream,
    _marker: PhantomData<S>,
}

impl<S: Schema> Client<S> {
    pub async fn connect(address: impl ToSocketAddrs) -> io::Result<Self> {
        let mut client = Self {
            tcp: TcpStream::connect(address).await?,
            _marker: PhantomData,
        };

        assert!(client.get_schema().await? == S::SCHEMA_NODE);

        Ok(client)
    }

    pub async fn get_schema(&mut self) -> io::Result<SchemaNode> {
        Request::GetSchema.write(&mut self.tcp).await?;

        SchemaNode::read(&mut self.tcp).await
    }

    pub async fn set_schema<N: Schema>(mut self, value: &N) -> io::Result<Client<N>> {
        Request::SetSchema(N::SCHEMA_NODE, value.value())
            .write(&mut self.tcp)
            .await?;

        Value::read(&N::SCHEMA_NODE, &mut self.tcp).await?;

        Ok(Client {
            tcp: self.tcp,
            _marker: PhantomData,
        })
    }

    pub async fn get(&mut self, schema: &SchemaNode<'_>) -> io::Result<Value<'static>> {
        Request::Get.write(&mut self.tcp).await?;

        Value::read(schema, &mut self.tcp).await
    }
}
