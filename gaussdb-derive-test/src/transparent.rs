use gaussdb::{Client, NoTls};
use gaussdb_types::{FromSql, ToSql};

#[test]
fn round_trip() {
    #[derive(FromSql, ToSql, Debug, PartialEq)]
    #[gaussdb(transparent)]
    struct UserId(i32);

    assert_eq!(
        Client::connect("user=gaussdb password=Gaussdb@123 host=localhost port=5433 dbname=postgres", NoTls)
            .unwrap()
            .query_one("SELECT $1::integer", &[&UserId(123)])
            .unwrap()
            .get::<_, UserId>(0),
        UserId(123)
    );
}
