use futures_util::FutureExt;
use openssl::ssl::{SslConnector, SslMethod};
use tokio::net::TcpStream;
use tokio_gaussdb::tls::TlsConnect;

use super::*;

#[cfg(feature = "runtime")]
use crate::MakeTlsConnector;

async fn smoke_test<T>(s: &str, tls: T)
where
    T: TlsConnect<TcpStream>,
    T::Stream: 'static + Send,
{
    let stream = TcpStream::connect("127.0.0.1:5433").await.unwrap();

    let builder = s.parse::<tokio_gaussdb::Config>().unwrap();
    let (client, connection) = builder.connect_raw(stream, tls).await.unwrap();

    let connection = connection.map(|r| r.unwrap());
    tokio::spawn(connection);

    let stmt = client.prepare("SELECT $1::INT4").await.unwrap();
    let rows = client.query(&stmt, &[&1i32]).await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 1);
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
async fn require() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    let ctx = builder.build();
    smoke_test(
        "user=ssl_user dbname=postgres sslmode=require",
        TlsConnector::new(ctx.configure().unwrap(), "localhost"),
    )
    .await;
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
async fn direct() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    set_postgresql_alpn(&mut builder).unwrap();
    let ctx = builder.build();
    smoke_test(
        "user=ssl_user dbname=postgres sslmode=require sslnegotiation=direct",
        TlsConnector::new(ctx.configure().unwrap(), "localhost"),
    )
    .await;
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
async fn prefer() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    let ctx = builder.build();
    smoke_test(
        "user=ssl_user dbname=postgres",
        TlsConnector::new(ctx.configure().unwrap(), "localhost"),
    )
    .await;
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
async fn scram_user() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    let ctx = builder.build();
    smoke_test(
        "user=scram_user password=password dbname=postgres sslmode=require",
        TlsConnector::new(ctx.configure().unwrap(), "localhost"),
    )
    .await;
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
async fn require_channel_binding_err() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    let ctx = builder.build();
    let connector = TlsConnector::new(ctx.configure().unwrap(), "localhost");

    let stream = TcpStream::connect("127.0.0.1:5433").await.unwrap();
    let builder = "user=pass_user password=password dbname=postgres channel_binding=require"
        .parse::<tokio_gaussdb::Config>()
        .unwrap();
    builder.connect_raw(stream, connector).await.err().unwrap();
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
async fn require_channel_binding_ok() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    let ctx = builder.build();
    smoke_test(
        "user=scram_user password=password dbname=postgres channel_binding=require",
        TlsConnector::new(ctx.configure().unwrap(), "localhost"),
    )
    .await;
}

#[tokio::test]
#[ignore] // GaussDB test environment doesn't support TLS/SSL connections
#[cfg(feature = "runtime")]
async fn runtime() {
    // TODO: GaussDB测试环境不支持TLS/SSL连接
    // 原因：测试环境中的GaussDB/OpenGauss实例未配置SSL证书
    // 影响：仅影响TLS连接测试，不影响实际TLS功能
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_ca_file("../test/server.crt").unwrap();
    let connector = MakeTlsConnector::new(builder.build());

    let (client, connection) = tokio_gaussdb::connect(
        "host=localhost port=5433 user=ssl_user password=password sslmode=require",
        connector,
    )
    .await
    .unwrap();
    let connection = connection.map(|r| r.unwrap());
    tokio::spawn(connection);

    let stmt = client.prepare("SELECT $1::INT4").await.unwrap();
    let rows = client.query(&stmt, &[&1i32]).await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<_, i32>(0), 1);
}
