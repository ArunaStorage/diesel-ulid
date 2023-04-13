# diesel-ulid

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-brightgreen.svg)](https://github.com/ArunaStorage/ArunaServer/blob/main/LICENSE-APACHE)
![CI](https://github.com/ArunaStorage/diesel-ulid/actions/workflows/push.yaml/badge.svg)
[![Codecov](https://codecov.io/github/ArunaStorage/diesel-ulid/coverage.svg?branch=main)](https://codecov.io/gh/ArunaStorage/ArunaServer)
[![Dependency status](https://deps.rs/repo/github/ArunaStorage/diesel-ulid/status.svg)](https://deps.rs/repo/github/ArunaStorage/diesel-ulid)
___


[Ulid](https://github.com/ulid/spec) mapping for [diesel-rs](https://github.com/diesel-rs/diesel). This crate contains custom mapping for the `Ulid` implementation from [rusty-ulid](https://github.com/huxi/rusty_ulid) to [diesel::sql_types::Uuid](https://docs.rs/diesel/latest/diesel/sql_types/struct.Uuid.html). With this adapter you can use these types in Diesel as regular Postgres UUID type.

## Import

```
cargo add diesel-ulid
```

or add:

```
diesel-ulid = 0.1.0
```

to your `Cargo.toml`.

## Usage

This is an adaptation of the [Getting started](https://diesel.rs/guides/getting-started) section from diesel-rs.

Assuming you have the following `schema.rs` file:

```rust

diesel::table! {
    posts (id) {
        id -> Uuid,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}
```

you could use diesel-ulid as follows:

```rust
use diesel::prelude::*;
use diesel-ulid::DieselUlid;

#[derive(Queryable)]
pub struct Post {
    pub id: DieselUlid,
    pub title: String,
    pub body: String,
    pub published: bool,
}
```

The [Postgres UUID](https://www.postgresql.org/docs/current/datatype-uuid.html) will be automatically mapped to a corresponding Ulid and vice-versa. This works because UUID and Ulid are both represented as 16 byte (128 bit) data struct. 

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.