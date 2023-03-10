use regex::Regex;

const RULES: &str = "https://rules2.clearurls.xyz/data.minify.json";

pub async fn clear_url(url: &str) -> Result<&str, Box<dyn std::error::Error>> {
    let rules = reqwest::get(RULES).await?.text().await?;
    let rules: serde_json::Value = serde_json::from_str(&rules)?;

    let providers = rules["providers"].clone();
    let providers = providers.as_object().unwrap();

    for provider in providers {
        let name = provider.0;
        let data = provider.1;

        let url_pattern = data
            .get("urlPattern")
            .unwrap()
            .as_str()
            .unwrap()
            .replace("\\", "");
        let re = Regex::new(&url_pattern)?;

        if re.is_match(url) {
            println!("found match: {name} - {}", re.as_str());
        }
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let url = "https://www.amazon.com/dp/exampleProduct/ref=sxin_0_pb?__mk_de_DE=ÅMÅŽÕÑ&keywords=tea&pd_rd_i=exampleProduct&pd_rd_r=8d39e4cd-1e4f-43db-b6e7-72e969a84aa5&pd_rd_w=1pcKM&pd_rd_wg=hYrNl&pf_rd_p=50bbfd25-5ef7-41a2-68d6-74d854b30e30&pf_rd_r=0GMWD0YYKA7XFGX55ADP&qid=1517757263&rnid=2914120011";
        let cleared = "https://www.amazon.com/dp/exampleProduct";

        match clear_url(url).await {
            Ok(val) => assert_eq!(val, cleared),
            Err(e) => println!("Error: {e}"),
        }
    }
}
