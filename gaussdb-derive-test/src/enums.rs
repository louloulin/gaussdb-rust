use crate::test_type;
use gaussdb::{error::DbError, Client, NoTls};
use gaussdb_test_helpers::*;
use gaussdb_types::{FromSql, ToSql, WrongType};
use std::error::Error;

#[test]
fn defaults() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    enum Foo {
        Bar,
        Baz,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.\"Foo\" AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    test_type(
        &mut conn,
        "\"Foo\"",
        &[(Foo::Bar, "'Bar'"), (Foo::Baz, "'Baz'")],
    );
}

#[test]
fn name_overrides() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(name = "mood")]
    enum Mood {
        #[gaussdb(name = "sad")]
        Sad,
        #[gaussdb(name = "ok")]
        Ok,
        #[gaussdb(name = "happy")]
        Happy,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute(
        "CREATE TYPE pg_temp.mood AS ENUM ('sad', 'ok', 'happy')",
        &[],
    )
    .unwrap();

    test_type(
        &mut conn,
        "mood",
        &[
            (Mood::Sad, "'sad'"),
            (Mood::Ok, "'ok'"),
            (Mood::Happy, "'happy'"),
        ],
    );
}

#[test]
fn rename_all_overrides() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(name = "mood", rename_all = "snake_case")]
    enum Mood {
        VerySad,
        #[gaussdb(name = "okay")]
        Ok,
        VeryHappy,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute(
        "CREATE TYPE pg_temp.mood AS ENUM ('very_sad', 'okay', 'very_happy')",
        &[],
    )
    .unwrap();

    test_type(
        &mut conn,
        "mood",
        &[
            (Mood::VerySad, "'very_sad'"),
            (Mood::Ok, "'okay'"),
            (Mood::VeryHappy, "'very_happy'"),
        ],
    );
}

#[test]
fn wrong_name() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    enum Foo {
        Bar,
        Baz,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    let err = conn.execute("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.source().unwrap().is::<WrongType>());
}

#[test]
fn extra_variant() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(name = "foo")]
    enum Foo {
        Bar,
        Baz,
        Buz,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    let err = conn.execute("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.source().unwrap().is::<WrongType>());
}

#[test]
fn missing_variant() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(name = "foo")]
    enum Foo {
        Bar,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    let err = conn.execute("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.source().unwrap().is::<WrongType>());
}

#[test]
fn allow_mismatch_enums() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(allow_mismatch)]
    enum Foo {
        Bar,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.\"Foo\" AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    let row = conn.query_one("SELECT $1::\"Foo\"", &[&Foo::Bar]).unwrap();
    assert_eq!(row.get::<_, Foo>(0), Foo::Bar);
}

#[test]
fn missing_enum_variant() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(allow_mismatch)]
    enum Foo {
        Bar,
        Buz,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.\"Foo\" AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    let err = conn
        .query_one("SELECT $1::\"Foo\"", &[&Foo::Buz])
        .unwrap_err();
    assert!(err.source().unwrap().is::<DbError>());
}

#[test]
fn allow_mismatch_and_renaming() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(name = "foo", allow_mismatch)]
    enum Foo {
        #[gaussdb(name = "bar")]
        Bar,
        #[gaussdb(name = "buz")]
        Buz,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('bar', 'baz', 'buz')", &[])
        .unwrap();

    let row = conn.query_one("SELECT $1::foo", &[&Foo::Buz]).unwrap();
    assert_eq!(row.get::<_, Foo>(0), Foo::Buz);
}

#[test]
fn wrong_name_and_allow_mismatch() {
    #[derive(Debug, ToSql, FromSql, PartialEq)]
    #[gaussdb(allow_mismatch)]
    enum Foo {
        Bar,
    }

    let mut conn = Client::connect(
        &get_test_conn_str(),
        NoTls,
    )
    .unwrap();
    conn.execute("CREATE TYPE pg_temp.foo AS ENUM ('Bar', 'Baz')", &[])
        .unwrap();

    let err = conn.query_one("SELECT $1::foo", &[&Foo::Bar]).unwrap_err();
    assert!(err.source().unwrap().is::<WrongType>());
}
