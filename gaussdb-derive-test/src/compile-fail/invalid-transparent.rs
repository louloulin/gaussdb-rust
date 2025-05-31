use gaussdb_types::{FromSql, ToSql};

#[derive(ToSql, Debug)]
#[gaussdb(transparent)]
struct ToSqlTransparentStruct {
    a: i32
}

#[derive(FromSql, Debug)]
#[gaussdb(transparent)]
struct FromSqlTransparentStruct {
    a: i32
}

#[derive(ToSql, Debug)]
#[gaussdb(transparent)]
enum ToSqlTransparentEnum {
    Foo
}

#[derive(FromSql, Debug)]
#[gaussdb(transparent)]
enum FromSqlTransparentEnum {
    Foo
}

#[derive(ToSql, Debug)]
#[gaussdb(transparent)]
struct ToSqlTransparentTwoFieldTupleStruct(i32, i32);

#[derive(FromSql, Debug)]
#[gaussdb(transparent)]
struct FromSqlTransparentTwoFieldTupleStruct(i32, i32);

fn main() {}
