use std::path::PathBuf;

use regex::Regex;
use tracing::{debug, info};
use url::Url;

const PROVIDERS_URL: &str = "https://rules2.clearurls.xyz/data.minify.json";
const PROVIDERS_FILE: &str = "./clearurl/data.minify.json";
const _HASH_URL: &str = "https://rules2.clearurls.xyz/rules.minify.hash";
const _HASH_PATH: &str = "./clearurl/rules.minify.hash";

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Remove tracking elements from url using the ClearURLs rules, which can be found
/// on the [github repo](https://github.com/ClearURLs/Addon)
pub async fn clear_url(url: &str) -> Result<String, Error> {
    /* tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init(); */

    let providers_path = PathBuf::from(PROVIDERS_FILE);

    // Download json file if not exists on disk or of last modified time is more
    // than 24h
    if !providers_path.is_file()
        || providers_path.metadata()?.modified()?.elapsed()?
            > std::time::Duration::from_secs(24 * 3600)
    {
        let response = reqwest::get(PROVIDERS_URL).await?;
        info!("Download rules files: {}", response.status());
        let text = response.text().await?;
        std::fs::write(&providers_path, text)?;
        info!("Rules file saved on disk at: {}", PROVIDERS_FILE);
    }

    let text = std::fs::read_to_string(&providers_path)?;
    let json: serde_json::Value = serde_json::from_str(&text)?;
    info!("Rules loaded from: {}", PROVIDERS_FILE);

    process_url(url, &json)
}

fn process_url(url: &str, json: &serde_json::Value) -> Result<String, Error> {
    let mut url = url.to_owned();

    let providers = json["providers"].as_object().unwrap();

    for provider in providers {
        let name = provider.0;
        let data = provider.1;

        // Get the pattern to match with the url domain name
        let url_pattern = data
            .get("urlPattern")
            .unwrap()
            .as_str()
            .unwrap()
            .replace('\\', "");
        let re = Regex::new(&url_pattern)?;

        // If url domain match the pattern
        if re.is_match(&url) {
            info!("URL: {url}");
            info!("found match: {name} - {}", re.as_str());

            //The completeProvider is a boolean, that determines if every URL that matches the urlPattern will be blocked.
            //If you want to specify rules, exceptions, and/or redirections, the value of completeProvider must be false.
            if let Some(complete) = data.get("completeProvider") {
                if complete.as_bool().unwrap() {
                    return Ok(String::new());
                }
            }

            // If any exceptions is found, the provider is skipped
            // if let Some(exceptions) = data.get("exceptions") {
            //     if exceptions.as_array().iter().any(|exception| {
            //         let pattern = exception[1].as_str().unwrap().replace("\\", "");
            //         println!("Exception: {pattern}");
            //         let re = Regex::new(&pattern).unwrap();
            //         re.is_match(&url)
            //     }) {
            //         println!("found exception: return without further processing");
            //         return Ok(String::from(url));
            //     }
            // }

            // If redirect found, recurse on target
            if let Some(redirections) = data.get("redirections") {
                for redir in redirections.as_array().unwrap() {
                    let pattern = redir[1].as_str().unwrap().replace('\\', "");
                    let re = Regex::new(&pattern)?;

                    if let Some(cap) = re.captures(&url) {
                        let redir_to = cap.get(1).unwrap().as_str();
                        info!("found redirect to: {redir_to}");
                        url = process_url(redir_to, json)?;
                    }
                }
            }

            // Explode query parameters to be checked against rules
            let mut parsed_url = Url::parse(&url)?;
            debug!("params:\n{:#?}", parsed_url);

            let mut params = vec![];
            let mut rules = vec![];
            let tags = ["rules", "referralMarketing"];

            for tag in tags {
                if let Some(data) = data.get(tag) {
                    rules.push(
                        data.as_array()
                            .unwrap()
                            .iter()
                            .map(|value| {
                                let rule_str = value.as_str().unwrap()/* .replace("(?:%3F)?", "") */;
                                Regex::new(&rule_str).unwrap()
                            })
                            .collect::<Vec<Regex>>(),
                    );
                }
            }
            let rules = rules.into_iter().flatten().collect::<Vec<Regex>>();
            debug!("RULES:\n{rules:#?}");

            // Check regular rules and referral marketing rules
            let pairs = parsed_url.query_pairs();

            'pair: for (key, value) in pairs {
                debug!("KEY: {}", &key);
                for re in &rules {
                    if re.is_match(&key.to_ascii_lowercase()) {
                        info!("match rule '{}' on '{}'", re.as_str(), &*key);
                        continue 'pair;
                    }
                }
                params.push((key.to_string(), value.to_string()));
            }

            let mut pairs = parsed_url.query_pairs_mut();
            pairs.clear();
            for param in params {
                pairs.append_pair(&param.0, &param.1);
            }
            drop(pairs);

            url = parsed_url.into();

            // Run raw rules against the URI string
            if let Some(raw_rules) = data.get("rawRules") {
                let rules = raw_rules
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| {
                        let rule_str = value.as_str().unwrap().replace('\\', "");
                        Regex::new(&rule_str).unwrap()
                    })
                    .collect::<Vec<Regex>>();

                for rule in rules {
                    if rule.is_match(&url) {
                        info!("raw rule match: {}", rule.as_str());
                    }
                    url = rule.replace(&url, "").to_string();
                }
            }

            info!("Processed URL: {}", url);
        }
    }

    url = url.trim_end_matches('?').trim_end_matches('/').into();
    info!("Cleaned URL: {}", url);

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn clean_amazon() {
        let url = "https://www.amazon.com/b/?node=226184&ref_=Oct_d_odnav_d_1077068_1&pd_rd_w=ZjwFQ&pf_rd_p=0f6f8a08-29ea-497e-8cb4-0ccf91422740&pf_rd_r=YMQ5XPAZHYHV77HCENY7&pd_rd_r=27c502f2-951f-4a8c-9478-381febc5e5bc&pd_rd_wg=NxaQ1";
        let cleared = "https://www.amazon.com/b/?node=226184";

        std::env::set_current_dir("../").unwrap();

        match clear_url(url).await {
            Ok(val) => assert_eq!(val, cleared),
            Err(e) => println!("Error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_filter() {
        let url = "https://twitter.com/CiloRanko/status/1478401918792011776?s=20&t=AVPOmNLtaozrA0Ccp6DyAw";
        let cleared = "https://twitter.com/CiloRanko/status/1478401918792011776";

        std::env::set_current_dir("../").unwrap();

        match clear_url(url).await {
            Ok(val) => assert_eq!(val, cleared),
            Err(e) => println!("Error: {e}"),
        }
    }

    #[tokio::test]
    async fn test_redir() {
        let url = "https://b23.tv/C0lw13z";
        let cleared = "https://www.bilibili.com/video/BV1GJ411x7h7/?p=1";

        std::env::set_current_dir("../").unwrap();

        match clear_url(url).await {
            Ok(val) => assert_eq!(val, cleared),
            Err(e) => println!("Error: {e}"),
        }
    }
}
