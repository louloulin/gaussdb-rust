use gaussdb_types::{FromSql, ToSql};

#[derive(FromSql)]
#[gaussdb(foo = "bar")]
struct Foo {
    a: i32,
}

#[derive(ToSql)]
#[gaussdb(foo = "bar")]
struct Bar {
    a: i32,
}

fn main() {}
