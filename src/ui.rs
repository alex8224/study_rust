extern crate hyper;

#[tokio::test]
async fn test_basic_httpclient() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use hyper::body::HttpBody as _;
    use hyper::Client;
    use tokio::io::{stdout, AsyncWriteExt as _};

    let client = Client::new();
    let uri = "http://ip138.com/iplookup.asp?ip=110.53.162.28&action=2".parse()?;
    let mut resp = client.get(uri).await?;
    println!("response {} {:?}", resp.status(), resp.headers());
    while let Some(chunk) = resp.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
        stdout().write_all(&chunk?).await?;
    }
    Ok(())
}
