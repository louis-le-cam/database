use std::borrow::Cow;

use database::{
    pipeline::{Stage, ValueOrPath},
    request::Request,
    schema::SchemaNode,
    value::Value,
};

#[tokio::main]
async fn main() {
    let tcp_listener = tokio::net::TcpListener::bind("localhost:8001")
        .await
        .unwrap();

    println!("listening on {}", tcp_listener.local_addr().unwrap());

    loop {
        let (tcp, _) = tcp_listener.accept().await.unwrap();

        println!("connection accepted on {}", tcp.local_addr().unwrap());

        tokio::task::spawn(async move {
            let (mut read, mut write) = tcp.into_split();

            let schema = SchemaNode::List(Box::new(SchemaNode::Product(vec![
                ("name".to_string(), SchemaNode::String),
                (
                    "location".to_string(),
                    SchemaNode::Sum(vec![
                        (
                            String::new(),
                            SchemaNode::Product(vec![
                                (String::new(), SchemaNode::U64),
                                (String::new(), SchemaNode::U64),
                            ]),
                        ),
                        (String::new(), SchemaNode::Unit),
                    ]),
                ),
            ])));

            let mut value = Value::List(vec![
                Value::Product(vec![
                    Value::String("test name".to_string()),
                    Value::Sum(
                        0,
                        Box::new(Value::Product(vec![Value::U64(114), Value::U64(217)])),
                    ),
                ]),
                Value::Product(vec![
                    Value::String("name 2".to_string()),
                    Value::Sum(1, Box::new(Value::Unit)),
                ]),
                Value::Product(vec![
                    Value::String("test name".to_string()),
                    Value::Sum(1, Box::new(Value::Unit)),
                ]),
                Value::Product(vec![
                    Value::String("test name".to_string()),
                    Value::Sum(1, Box::new(Value::Unit)),
                ]),
                Value::Product(vec![
                    Value::String("name 3".to_string()),
                    Value::Sum(1, Box::new(Value::Unit)),
                ]),
            ]);

            loop {
                match Request::read(&schema, &mut read).await.unwrap() {
                    Request::GetSchema => {
                        schema.write(&mut write).await.unwrap();
                    }
                    Request::Get { path } => {
                        value
                            .scope_ref(path.as_ref())
                            .unwrap()
                            .write(&mut write)
                            .await
                            .unwrap();
                    }
                    Request::Pipeline(stages) => {
                        let mut stage_schema = Cow::Borrowed(&schema);
                        // TODO: do not clone here, this is extremly bad
                        let mut stage_value: Cow<'_, Value> = Cow::Owned(value.clone());

                        for stage in stages.into_owned().into_iter() {
                            match stage {
                                Stage::Input(input_schema, input_value) => {
                                    stage_schema = Cow::Owned(input_schema);
                                    stage_value = Cow::Owned(input_value);
                                }
                                Stage::Get { path } => {
                                    stage_schema = Cow::Borrowed(schema.scope_ref(&path).unwrap());
                                    // TODO: do not clone here, this is extremly bad
                                    stage_value =
                                        Cow::Owned(value.scope_ref(&path).unwrap().clone())
                                }
                                Stage::Set {
                                    destination: set_destination,
                                    value: set_value,
                                } => {
                                    // TODO: maybe prevent cloning here
                                    let old_stage_value = stage_value.as_ref().clone();

                                    *value.scope_mut(&set_destination).unwrap() = match set_value {
                                        ValueOrPath::Value(value) => value,
                                        ValueOrPath::Path(path) => {
                                            old_stage_value.scope(&path).unwrap()
                                        }
                                    }
                                }
                                Stage::Filter { path, condition } => {
                                    let mut owned_stage_value = stage_value.into_owned();

                                    let Some(Value::List(values)) =
                                        owned_stage_value.scope_mut(&path)
                                    else {
                                        panic!()
                                    };

                                    *values = std::mem::replace(values, Vec::new())
                                        .into_iter()
                                        .filter(|value| match &condition {
                                            database::pipeline::Condition::Equal {
                                                schema: _,
                                                lhs,
                                                rhs,
                                            } => {
                                                let lhs = match lhs {
                                                    ValueOrPath::Value(value) => value,
                                                    ValueOrPath::Path(path) => {
                                                        value.scope_ref(&path).unwrap()
                                                    }
                                                };

                                                let rhs = match rhs {
                                                    ValueOrPath::Value(value) => value,
                                                    ValueOrPath::Path(path) => {
                                                        value.scope_ref(&path).unwrap()
                                                    }
                                                };

                                                lhs == rhs
                                            }
                                        })
                                        .collect::<Vec<Value>>();

                                    stage_value = Cow::Owned(owned_stage_value);
                                }
                            }
                        }

                        stage_value.write(&mut write).await.unwrap();
                    }
                }
            }
        });
    }
}
