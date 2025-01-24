use std::marker::PhantomData;

use database::{schema, Client, Schema, Server};

schema! {
    struct User {
        name: String,
        location: Option<(f32,f32)>,
    }
}

enum Expression<T> {
    Chain(Box<(Expression<()>, Expression<T>)>, PhantomData<T>),
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).is_some_and(|arg| arg == "client") {
        let mut client = Client::<Vec<User>>::connect(("localhost", 28493))
            .await
            .unwrap();

        client.get(&Vec::<User>::SCHEMA_NODE).await.unwrap();

        // let t = "dqdq".equal("dqzd");
        // let t = StringMut::from_path((&[0, 1, 2]).into()).equal("dqzd");
        // let t = StringRef::from_path((&[0, 1, 2]).into())
        //     .equal(StringRef::from_path((&[0, 1, 2]).into()));

        // let t = 32u32.equal(Uint64Ref::from_path((&[0, 1, 2]).into()));

        // let t = StringMut::from_path((&[0, 1, 2]).into()).set("dqdq")
        //     & Uint32Ref::from_path((&[0, 1, 2]).into()).equal(2424);

        // Chain(Box((
        //     Set(Box((Vec([0, 1, 2]), Value(String, String("dqdq"))))),
        //     Eq(Box((Path(Vec([0, 1, 2])), Value(String, String("dqzd"))))),
        // )));

        // let t = 2824.equal();

        // client.query(|user: <User as Schema>::Mut| {
        //     user.name.eq("<name-here>")
        // }).await.uwnrap() /* bool */;

        // // Find user by name
        // client.query(|users /*Vec<User>::Mut*/| {
        //     users.find(|user /*User::Mut*/| user.name.eq("<name-here>"))
        // }).await.unwrap() /*Option<User>*/;

        // // Change user location
        // client.query(|users /*Vec<User>::Mut*/| {
        //     users.find(|user| user.name().eq("<name-here>"))
        //         .if_some(|user| user.location().set(Some((83.8, -23.1)))) & ()
        // }).await.unwrap() /*()*/;

        // // Create user
        // client.query(|users /*Vec<User>::Mut*/| {
        //     users.push(User { name: "<name-here>".to_string(), location: None })
        // }).await.unwrap() /*()*/;

        // // Fuzzy user search
        // client.query(|users /*Vec<User>::Mut*/| {
        //     users.sort_by_key(|user| todo!())
        // }).await.unwrap() /* Vec<User> */;

        // // Clear users
        // client.query(|users /*Vec<User>::Mut*/| {
        //     users.set(&[])
        // }).await.unwrap() /* Vec<User> */;
    } else {
        Server::new().listen(("localhost", 28493)).await.unwrap();
    }
}
