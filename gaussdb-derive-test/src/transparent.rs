use gaussdb::{Client, NoTls};
use gaussdb_types::{FromSql, ToSql};
use gaussdb_test_helpers::*;

#[test]
fn round_trip() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[gaussdb(transparent)]
    struct UserId(i32);

    assert_eq!(
        Client::connect(
            &get_test_conn_str(),
            NoTls
        )
        .unwrap()
        .query_one("SELECT $1::integer", &[&UserId(123)])
        .unwrap()
        .get::<_, UserId>(0),
        UserId(123)
    );
}
