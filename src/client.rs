use std::{io, marker::PhantomData};

use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, ToSocketAddrs},
};

use crate::{Expression, FromPath, Schema, SchemaNode, Scope};

pub struct Client<S: Schema + Send + Sync> {
    tcp: TcpStream,
    _marker: PhantomData<S>,
}

impl<S: Schema + Send + Sync> Client<S> {
    pub async fn connect(address: impl ToSocketAddrs) -> io::Result<Self> {
        Ok(Self {
            tcp: TcpStream::connect(address).await?,
            _marker: PhantomData,
        })
    }

    pub async fn get_schema(&mut self) -> io::Result<SchemaNode> {
        self.tcp.write_u8(0).await?;

        SchemaNode::read(&mut self.tcp).await
    }

    pub async fn set<NewS: Schema + Send + Sync>(
        mut self,
        value: NewS,
    ) -> io::Result<Client<NewS>> {
        self.tcp.write_u8(1).await?;

        NewS::write_schema(&mut self.tcp).await?;
        value.write_value(&mut self.tcp).await?;

        Ok(Client {
            tcp: self.tcp,
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

        self.tcp.write_u8(2).await?;

        expression.write(&mut self.tcp).await?;

        E::Target::read_value(&mut self.tcp).await
    }
}
