use reqwest::cookie::{self};
use std::sync::Arc;

pub fn read_secret_in_directory() -> Option<Vec<String>> {
    println!("Reading cookies from ./secret");
    match std::fs::read_to_string("./secret") {
        Ok(content) => {
            let lines: Vec<String> = content
                .lines()
                .map(|line| line.trim().to_string())
                .collect();
            if lines.len() >= 2 {
                println!("Cookies read successfully from ./secret");
                Some(lines)
            } else {
                eprintln!("Invalid cookie format in ./secret");
                None
            }
        }
        Err(_) => None,
    }
}

pub fn write_secret_in_directory<C>(cookie_store: Arc<C>)
where
    C: cookie::CookieStore + 'static,
{
    let cookies = cookie_store
        .cookies(&url::Url::parse("https://api.vrchat.cloud").expect("Invalid URL"))
        .unwrap();

    let auth_cookie = cookies
        .to_str()
        .unwrap()
        .split(';')
        .find(|cookie| cookie.trim().starts_with("auth="))
        .map(|cookie| cookie.trim().to_string())
        .unwrap_or_default();

    let two_factor_cookie = cookies
        .to_str()
        .unwrap()
        .split(';')
        .find(|cookie| cookie.trim().starts_with("twoFactorAuth="))
        .map(|cookie| cookie.trim().to_string())
        .unwrap_or_default();

    if let Err(e) = std::fs::write(
        "./secret",
        format!("{}\n{}", auth_cookie, two_factor_cookie),
    ) {
        eprintln!("Failed to save cookies: {}", e);
    } else {
        println!("Cookies saved successfully to ./secret");
    }
}
