use crate::authenticate;
use reqwest::Client;

pub async fn create_client() -> Result<Client, reqwest::Error> {
    let authenticate = authenticate::auth();
    let proxies = format!("http://{}", authenticate);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .proxy(reqwest::Proxy::all(proxies)?)
        .build()?;
    Ok(client)
}

#[cfg(test)]
mod tests {
    use crate::proxied_reqwest::create_client;
    #[tokio::test()]
    #[warn(unused_must_use)]
    async fn test_api_call() {
        let client = create_client().await.unwrap();
        let response = client.get("https://ipinfo.io/json").send().await.unwrap();
        dbg!(response.text().await.unwrap());
    }
}
