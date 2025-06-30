# auto_builder_derive

A procedural macro to generate ergonomic builder patterns for Rust structs.

## Features

- **Custom setter names**: `#[builder(setter = "with_field")]`
- **Default values**: `#[builder(default = ...)]`
- **Optional fields**: `Option<T>` fields are optional in the builder
- **Vec ergonomics**: `add_item`, `add_items`, `set_items` (customizable)
- **Skipped fields**: `#[builder(skip)]` or `#[builder(skip = ...)]`
- **Chaining and Result-based build**

## Example Usage

```rust
use auto_builder::AutoBuilder;

#[derive(AutoBuilder, Debug)]
struct User {
    // Required field with custom setter
    #[builder(setter = "with_name")]
    name: String,

    // Optional field (Option<T> is always optional in the builder)
    nickname: Option<String>,

    // Field with a default value
    #[builder(default = 18)]
    age: u32,

    // Vec field with all three custom methods
    #[builder(setter_push = "add_role", setter_push_many = "add_roles", setter_set = "set_roles")]
    roles: Vec<String>,

    // Skipped field, always set to 42 in the built struct
    #[builder(skip = 42)]
    secret_code: u32,
}

fn main() {
    // Build a user with all features
    let user = UserBuilder::new()
        .with_name("Alice".to_string())
        .add_role("admin".to_string())
        .add_roles(vec!["editor".to_string(), "user".to_string()])
        .set_roles(vec!["root".to_string()]) // replaces all roles
        .age(30) // uses the default if not set
        .build()
        .unwrap();
    println!("user: {:?}", user);

    // Build a user with only required fields
    let user2 = UserBuilder::new()
        .with_name("Bob".to_string())
        .build()
        .unwrap();
    println!("user2: {:?}", user2);

    // Error handling: missing required field
    let result = UserBuilder::new().build();
    assert!(result.is_err());
}
```

## Attribute Reference

- `#[builder(setter = "...")]`: Custom setter name for a field.
- `#[builder(default = ...)]`: Provide a default value for a field.
- `#[builder(skip)]` or `#[builder(skip = ...)]`: Skip a field in the builder you can set default value for skip.
- `#[builder(setter_push = "...")]`, `#[builder(setter_push_many = "...")]`, `#[builder(setter_set = "...")]`: Custom names for Vec field methods.
