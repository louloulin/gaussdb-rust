use futures_util::{join, FutureExt};
use std::time::Duration;
use tokio::time;
use tokio_gaussdb::error::SqlState;
use tokio_gaussdb::{Client, NoTls};

async fn connect(s: &str) -> Client {
    let (client, connection) = tokio_gaussdb::connect(s, NoTls).await.unwrap();
    let connection = connection.map(|e| e.unwrap());
    tokio::spawn(connection);

    client
}

async fn smoke_test(s: &str) {
    let client = connect(s).await;

    let stmt = client.prepare("SELECT $1::INT").await.unwrap();
    let rows = client.query(&stmt, &[&1i32]).await.unwrap();
    assert_eq!(rows[0].get::<_, i32>(0), 1i32);
}

#[tokio::test]
#[ignore] // FIXME doesn't work with our docker-based tests :(
async fn unix_socket() {
    smoke_test("host=/var/run/postgresql port=5433 user=postgres").await;
}

#[tokio::test]
async fn tcp() {
    smoke_test("host=localhost port=5433 user=gaussdb password=Gaussdb@123 dbname=postgres").await;
}

#[tokio::test]
async fn multiple_hosts_one_port() {
    smoke_test(
        "host=foobar.invalid,localhost port=5433 user=gaussdb password=Gaussdb@123 dbname=postgres",
    )
    .await;
}

#[tokio::test]
async fn multiple_hosts_multiple_ports() {
    smoke_test("host=foobar.invalid,localhost port=5432,5433 user=gaussdb password=Gaussdb@123 dbname=postgres").await;
}

#[tokio::test]
async fn wrong_port_count() {
    tokio_gaussdb::connect(
        "host=localhost port=5433,5433 user=gaussdb password=Gaussdb@123 dbname=postgres",
        NoTls,
    )
    .await
    .err()
    .unwrap();
}

#[tokio::test]
async fn target_session_attrs_ok() {
    smoke_test("host=localhost port=5433 user=gaussdb password=Gaussdb@123 dbname=postgres target_session_attrs=read-write").await;
}

#[tokio::test]
async fn target_session_attrs_err() {
    tokio_gaussdb::connect(
        "host=localhost port=5433 user=gaussdb password=Gaussdb@123 dbname=postgres target_session_attrs=read-write
         options='-c default_transaction_read_only=on'",
        NoTls,
    )
    .await
    .err()
    .unwrap();
}

#[tokio::test]
async fn host_only_ok() {
    let _ = tokio_gaussdb::connect(
        "host=localhost port=5433 user=gaussdb dbname=postgres password=Gaussdb@123",
        NoTls,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn hostaddr_only_ok() {
    let _ = tokio_gaussdb::connect(
        "hostaddr=127.0.0.1 port=5433 user=gaussdb dbname=postgres password=Gaussdb@123",
        NoTls,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn hostaddr_and_host_ok() {
    let _ = tokio_gaussdb::connect(
        "hostaddr=127.0.0.1 host=localhost port=5433 user=gaussdb dbname=postgres password=Gaussdb@123",
        NoTls,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn hostaddr_host_mismatch() {
    let _ = tokio_gaussdb::connect(
        "hostaddr=127.0.0.1,127.0.0.2 host=localhost port=5433 user=gaussdb dbname=postgres password=Gaussdb@123",
        NoTls,
    )
    .await
    .err()
    .unwrap();
}

#[tokio::test]
async fn hostaddr_host_both_missing() {
    let _ = tokio_gaussdb::connect(
        "port=5433 user=gaussdb dbname=postgres password=Gaussdb@123",
        NoTls,
    )
    .await
    .err()
    .unwrap();
}

#[tokio::test]
async fn cancel_query() {
    let client =
        connect("host=localhost port=5433 user=gaussdb password=Gaussdb@123 dbname=postgres").await;

    let cancel_token = client.cancel_token();
    let cancel = cancel_token.cancel_query(NoTls);
    let cancel = time::sleep(Duration::from_millis(100)).then(|()| cancel);

    let sleep = client.batch_execute("SELECT pg_sleep(100)");

    match join!(sleep, cancel) {
        (Err(ref e), Ok(())) if e.code() == Some(&SqlState::QUERY_CANCELED) => {}
        t => panic!("unexpected return: {:?}", t),
    }
}
