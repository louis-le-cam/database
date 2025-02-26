# Experimental typed database engine (WIP)

This database is currently work-in-progress, it is certainly very slow and lacks lot of features

## Model your data correctly

This database implements [algebraic data types](https://en.wikipedia.org/wiki/Algebraic_data_type) (like rust)
to make your schema closer to your actual values, a typesystem that correctly represent data can avoid lots of mistakes

```rust
#[derive(Shape)]
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
  Circle(f32),
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
    name: "some user",
    favorite_shape: Some(Shape::Rectangle {
      width: 32.0,
      height: 16.8,
    }),
  })
});
```

## Next steps
- [x] Add more data types
  - [x] integers of different size, signed and unsigned
  - [x] floating points
- [x] Add macro to implements `Schema` trait for user-defined types
  - [x] unit struct
  - [x] struct with named fields
  - [x] tuple struct
  - [x] enum
  - [x] #[derive] macro instead of macro_rules!
- [x] Add a way to modify the schema from the client
- [ ] Add more expressions
  - [ ] simple binary operators: && || + - * / %
  - [ ] list operators: push insert remove
- [ ] Add lazy iterators
- [ ] Find a better way to represent data, and to have it partially loadeable in memory
- [ ] Optimize the expression evaluation
- [ ] Save the data to the filesystem
- [ ] Add partial values to save network traffic
- [x] Add other types of collections
  - [x] hash map
  - [x] slot map that can simulate sql-like relations, with a new-type key
- [ ] Some sort of indexing
