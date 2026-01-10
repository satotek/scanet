use reqwest::Client;

/// Look up vendor name from MAC address using macvendors.com API
pub async fn lookup_vendor_online(mac: &str) -> Option<String> {
    let client = Client::new();
    let url = format!("https://api.macvendors.com/{}", mac);

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                response.text().await.ok().filter(|s| !s.is_empty())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
