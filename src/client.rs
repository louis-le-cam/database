use std::{io, marker::PhantomData};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{request_discriminant, Expression, FromPath, Schema, SchemaNode, Scope};

pub struct Client<S: Schema + Send + Sync, St: AsyncReadExt + AsyncWriteExt + Unpin + Send> {
    stream: St,
    _marker: PhantomData<S>,
}

impl<S: Schema + Send + Sync, St: AsyncReadExt + AsyncWriteExt + Unpin + Send> Client<S, St> {
    pub async fn new(stream: St) -> io::Result<Self> {
        Ok(Self {
            stream,
            _marker: PhantomData,
        })
    }

    pub async fn new_tcp(address: impl ToSocketAddrs) -> io::Result<Client<S, TcpStream>> {
        Ok(Client {
            stream: TcpStream::connect(address).await?,
            _marker: PhantomData,
        })
    }

    pub async fn get_schema(&mut self) -> io::Result<SchemaNode> {
        self.stream
            .write_u8(request_discriminant::GET_SCHEMA)
            .await?;

        SchemaNode::read(&mut self.stream).await
    }

    pub async fn set<NewS: Schema + Send + Sync>(
        mut self,
        value: NewS,
    ) -> io::Result<Client<NewS, St>> {
        self.stream.write_u8(request_discriminant::SET).await?;

        NewS::write_schema(&mut self.stream).await?;
        value.write_value(&mut self.stream).await?;

        Ok(Client {
            stream: self.stream,
            _marker: PhantomData,
        })
    }

    pub async fn query<E: Expression>(
        &mut self,
        query: impl FnOnce(S::Expression) -> E,
    ) -> io::Result<E::Target> {
        Scope::create();
        let expression = (query)(<S::Expression as FromPath>::from_path(vec![0]));
        Scope::delete();

        self.stream.write_u8(request_discriminant::QUERY).await?;

        expression.write(&mut self.stream).await?;

        E::Target::read_value(&mut self.stream).await
    }
}
