# Experimental typed database engine (WIP)

This database is currently work-in-progress, it is certainly very slow and lacks lot of features

## Model your data correctly

This database implements [algebraic data types](https://en.wikipedia.org/wiki/Algebraic_data_type) (like rust)
to make your schema closer to your actual values, a typesystem that correctly represent data can avoid lots of mistakes

```rust
// `#[derive(Schema)]` is not yet implemented but there is a prototype macro_rules! in `./prototype2/schema.rs`

#[derive(Schema)]
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
  Circle {
    radius: f32,
  }
}

#[derive(Schema)]
struct User {
  name: String,
  favorite_shape: Option<Shape>,
}
```

## Query written like code

Queries are written just like any code in your favorite language
(currently there is only a rust client, but I plan to add a typescript client once the server is more complete)

```rust
client.query(|users| {
  users.filter(|user| user.name.equal("some user"))
});

client.query(|users| {
  // `push` is not yet implement
  users.push(User {
    name: "some user"
  })
});
```

## Next steps
- [ ] Add macro to implements `Schema` trait for user-defined types
- [ ] Add more expressions
  - [ ] simple binary operators: && || + - * / %
  - [ ] list operators: push insert remove
- [ ] Add lazy iterators
- [ ] Find a better way to represent data, and to have it partially loadeable in memory
- [ ] Optimize the expression evaluation
- [ ] Save the data to the filesystem
- [ ] Add partial values to save network traffic
- [ ] Add other types of collections
  - [ ] hash map
  - [ ] slot map that can simulate sql-like relations, with a new-type key
- [ ] Some sort of indexing
