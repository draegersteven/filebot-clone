use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct MovieMatch {
    pub title: String,
    pub year: Option<u16>,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct MovieQuery {
    pub title: String,
    pub year: Option<u16>,
    pub language: String,
}

pub trait HttpClient {
    fn get_json(&self, url: &str, query: &[(&str, String)], bearer: &str) -> Result<String, Box<dyn Error>>;
}

pub struct CurlHttpClient;
impl HttpClient for CurlHttpClient {
    fn get_json(&self, url: &str, query: &[(&str, String)], bearer: &str) -> Result<String, Box<dyn Error>> {
        let mut full_url = url.to_string();
        if !query.is_empty() {
            full_url.push('?');
            full_url.push_str(&query.iter().map(|(k,v)| format!("{}={}", url_encode(k), url_encode(v))).collect::<Vec<_>>().join("&"));
        }
        let output = std::process::Command::new("curl")
            .args(["-sS", "-H", &format!("Authorization: Bearer {bearer}"), &full_url])
            .output()?;
        if !output.status.success() {
            return Err(format!("curl failed with status {}", output.status).into());
        }
        Ok(String::from_utf8(output.stdout)?)
    }
}

fn url_encode(input: &str) -> String {
    input
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b as char],
            b' ' => vec!['+'],
            _ => format!("%{:02X}", b).chars().collect(),
        })
        .collect()
}

pub fn match_movie(client: &dyn HttpClient, key: &str, query: &MovieQuery) -> Result<Option<MovieMatch>, Box<dyn Error>> {
    let mut params = vec![("query", query.title.clone()), ("language", query.language.clone())];
    if let Some(year) = query.year { params.push(("year", year.to_string())); }
    let payload = client.get_json("https://api.themoviedb.org/3/search/movie", &params, key)?;

    let title = extract_json_string(&payload, "title");
    if title.is_none() {
        return Ok(None);
    }
    let title = title.unwrap();
    let year = extract_json_string(&payload, "release_date")
        .and_then(|d| d.get(0..4).and_then(|y| y.parse::<u16>().ok()));
    let pop = extract_json_number(&payload, "popularity").unwrap_or(0.0);
    let mut score = (pop.min(100.0) as f32) / 100.0;
    if title.eq_ignore_ascii_case(&query.title) { score += 0.5; }
    if query.year.is_some() && query.year == year { score += 0.5; }

    Ok(Some(MovieMatch { title, year, score: score.min(1.0) }))
}

fn extract_json_string(payload: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let idx = payload.find(&needle)?;
    let rest = &payload[idx + needle.len()..];
    let first_quote = rest.find('"')?;
    let rem = &rest[first_quote + 1..];
    let end_quote = rem.find('"')?;
    Some(rem[..end_quote].to_string())
}

fn extract_json_number(payload: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{key}\"");
    let idx = payload.find(&needle)?;
    let rest = &payload[idx + needle.len()..];
    let colon = rest.find(':')?;
    let n = rest[colon + 1..].trim_start();
    let end = n.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(n.len());
    n[..end].parse().ok()
}

pub fn required_tmdb_key(arg: &Option<String>) -> Result<String, Box<dyn Error>> {
    arg.clone().or_else(|| std::env::var("TMDB_KEY").ok()).ok_or_else(|| "TMDB key missing".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct MockClient { response: String, seen_query: RefCell<Vec<(String, String)>> }
    impl HttpClient for MockClient {
        fn get_json(&self, _url: &str, query: &[(&str, String)], _bearer: &str) -> Result<String, Box<dyn Error>> {
            self.seen_query.borrow_mut().extend(query.iter().map(|(k,v)| (k.to_string(), v.clone())));
            Ok(self.response.clone())
        }
    }
    fn fixture(name: &str) -> String { std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap() }
    fn query() -> MovieQuery { MovieQuery { title: "The Matrix".into(), year: Some(1999), language: "de-DE".into() } }

    #[test] fn t1(){assert_eq!(match_movie(&MockClient{response:fixture("tmdb_search_ok.json"),seen_query:RefCell::new(vec![])},"k",&query()).unwrap().unwrap().title,"The Matrix");}
    #[test] fn t2(){assert_eq!(match_movie(&MockClient{response:fixture("tmdb_search_ok.json"),seen_query:RefCell::new(vec![])},"k",&query()).unwrap().unwrap().year,Some(1999));}
    #[test] fn t3(){assert!(match_movie(&MockClient{response:fixture("tmdb_search_ok.json"),seen_query:RefCell::new(vec![])},"k",&query()).unwrap().unwrap().score>0.9);}
    #[test] fn t4(){assert!(match_movie(&MockClient{response:fixture("tmdb_search_no_results.json"),seen_query:RefCell::new(vec![])},"k",&query()).unwrap().is_none());}
    #[test] fn t5(){assert!(match_movie(&MockClient{response:fixture("tmdb_search_year_miss.json"),seen_query:RefCell::new(vec![])},"k",&query()).unwrap().unwrap().score<1.0);}
    #[test] fn t6(){let c=MockClient{response:fixture("tmdb_search_ok.json"),seen_query:RefCell::new(vec![])};let _=match_movie(&c,"k",&query()).unwrap();assert!(c.seen_query.borrow().iter().any(|(k,_)|k=="year"));}
    #[test] fn t7(){let c=MockClient{response:fixture("tmdb_search_ok.json"),seen_query:RefCell::new(vec![])};let _=match_movie(&c,"k",&query()).unwrap();assert!(c.seen_query.borrow().iter().any(|(k,v)|k=="language"&&v=="de-DE"));}
    #[test] fn t8(){let c=MockClient{response:fixture("tmdb_search_ok.json"),seen_query:RefCell::new(vec![])};let q=MovieQuery{title:"X".into(),year:None,language:"en-US".into()};let _=match_movie(&c,"k",&q).unwrap();assert!(!c.seen_query.borrow().iter().any(|(k,_)|k=="year"));}
    #[test] fn t9(){assert!(required_tmdb_key(&Some("abc".into())).is_ok());}
    #[test] fn t10(){std::env::set_var("TMDB_KEY","x");assert_eq!(required_tmdb_key(&None).unwrap(),"x");std::env::remove_var("TMDB_KEY");}
}
