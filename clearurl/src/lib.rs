use regex::Regex;
use url::Url;

const RULES: &str = "https://rules2.clearurls.xyz/data.minify.json";

/// Remove tracking elements from url using the ClearURLs rules, which can be found
/// on the github repo https://github.com/ClearURLs/Addon
pub async fn clear_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let rules = reqwest::get(RULES).await?.text().await?;
    let rules: serde_json::Value = serde_json::from_str(&rules)?;

    process_url(url, &rules)
}

fn process_url(url: &str, rules: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut url = url.to_owned();

    let providers = rules["providers"].clone();
    let providers = providers.as_object().unwrap();

    for provider in providers {
        let name = provider.0;
        let data = provider.1;

        // Get the pattern to match with the url domain name
        let url_pattern = data
            .get("urlPattern")
            .unwrap()
            .as_str()
            .unwrap()
            .replace("\\", "");
        let re = Regex::new(&url_pattern)?;

        // If url domain match the pattern
        if re.is_match(&url) {
            println!("found match: {name} - {}", re.as_str());

            //The completeProvider is a boolean, that determines if every URL that matches the urlPattern will be blocked.
            //If you want to specify rules, exceptions, and/or redirections, the value of completeProvider must be false.
            if let Some(complete) = data.get("completeProvider") {
                if complete.as_bool().unwrap() {
                    return Ok(String::from(""));
                }
            }

            // If any exceptions is found, the provider is skipped
            /* if let Some(exceptions) = data.get("exceptions") {
                if exceptions.as_array().iter().any(|exception| {
                    let pattern = exception[1].as_str().unwrap().replace("\\", "");
                    let re = Regex::new(&pattern).unwrap();
                    re.is_match(&url)
                }) {
                    println!("found exception: return without further processing");
                    return Ok(String::from(url));
                }
            } */

            // If redirect found, recurse on target
            if let Some(redirections) = data.get("redirections") {
                for redir in redirections.as_array().unwrap() {
                    let pattern = redir[1].as_str().unwrap().replace("\\", "");
                    let re = Regex::new(&pattern)?;

                    if let Some(cap) = re.captures(&url) {
                        let redir_to = cap.get(1).unwrap().as_str();
                        println!("found redirect to: {redir_to}");
                        url = process_url(redir_to, rules)?;
                    }
                }
            }

            // Explode query parameters to be checked against rules
            let mut parsed_url = Url::parse(&url)?;
            // println!("params:\n{:#?}", parsed_url);
            let mut params = vec![];

            // Check regular rules and referral marketing rules
            if let Some(rules) = data.get("rules") {
                let pairs = parsed_url.query_pairs();
                let rules = rules
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap())
                    .collect::<Vec<&str>>();

                println!("before: ");
                for pair in pairs {
                    println!("({}, {})", pair.0, pair.1);
                }

                println!("rules:\n{:#?}", rules);
                let re_rules = rules
                    .iter()
                    .map(|r| Regex::new(r).unwrap())
                    .collect::<Vec<Regex>>();

                'pair: for (key, value) in pairs {
                    println!("KEY: {}", &*key);
                    for rule in &re_rules {
                        println!("RULE: {}", rule.as_str());
                        if rule.is_match(&*key.to_ascii_lowercase()) {
                            println!("match rule '{}' on '{}'", rule.as_str(), &*key);
                            continue 'pair;
                        }
                    }
                    params.push((key.to_string(), value.to_string()));
                }

                println!("after:\n{:#?}", params);
            }

            let mut pairs = parsed_url.query_pairs_mut();
            pairs.clear();
            for param in params {
                pairs.append_pair(&param.0, &param.1);
            }
            drop(pairs);

            url = parsed_url.into();
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
