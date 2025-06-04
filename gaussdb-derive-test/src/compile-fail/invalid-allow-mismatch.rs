use gaussdb_types::{FromSql, ToSql};

#[derive(ToSql, Debug)]
#[gaussdb(allow_mismatch)]
struct ToSqlAllowMismatchStruct {
    a: i32,
}

#[derive(FromSql, Debug)]
#[gaussdb(allow_mismatch)]
struct FromSqlAllowMismatchStruct {
    a: i32,
}

#[derive(ToSql, Debug)]
#[gaussdb(allow_mismatch)]
struct ToSqlAllowMismatchTupleStruct(i32, i32);

#[derive(FromSql, Debug)]
#[gaussdb(allow_mismatch)]
struct FromSqlAllowMismatchTupleStruct(i32, i32);

#[derive(FromSql, Debug)]
#[gaussdb(transparent, allow_mismatch)]
struct TransparentFromSqlAllowMismatchStruct(i32);

#[derive(FromSql, Debug)]
#[gaussdb(allow_mismatch, transparent)]
struct AllowMismatchFromSqlTransparentStruct(i32);

fn main() {}
