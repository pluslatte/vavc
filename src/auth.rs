use reqwest::{cookie::CookieStore, header::HeaderValue};
use std::{str::FromStr, sync::Arc};
use vrchatapi::{
    apis,
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode},
};

use crate::{
    read_user_input,
    secret::{read_secret_in_directory, write_secret_in_directory},
};

pub async fn get_new_auth_cookie(username: Option<String>, password: Option<String>) {
    let username = match username {
        Some(name) => name,
        None => read_user_input("Enter your username: "),
    };
    let password = match password {
        Some(pass) => pass,
        None => read_user_input("Enter your password: "),
    };

    let jar = Arc::new(reqwest::cookie::Jar::default());

    let config = apis::configuration::Configuration {
        basic_auth: Some((username, Some(password))),
        user_agent: Some(String::from("my-rust-client/1.0.0")),
        client: reqwest::Client::builder()
            .cookie_provider(jar.clone())
            .build()
            .unwrap(),
        ..Default::default()
    };

    match apis::authentication_api::get_current_user(&config)
        .await
        .unwrap()
    {
        vrchatapi::models::EitherUserOrTwoFactor::CurrentUser(user) => {
            println!("Username: {}", user.username.unwrap());
        }
        vrchatapi::models::EitherUserOrTwoFactor::RequiresTwoFactorAuth(auth_required) => {
            if auth_required
                .requires_two_factor_auth
                .contains(&String::from("emailOtp"))
            {
                let code = read_user_input("Enter the 2FA code sent to your email: ");
                if let Err(e) = apis::authentication_api::verify2_fa_email_code(
                    &config,
                    TwoFactorEmailCode::new(code),
                )
                .await
                {
                    eprintln!("Failed to verify 2FA code: {}", e);
                }
            } else {
                let code = read_user_input("Enter your 2FA code: ");
                if let Err(e) =
                    apis::authentication_api::verify2_fa(&config, TwoFactorAuthCode::new(code))
                        .await
                {
                    eprintln!("Failed to verify 2FA code: {}", e);
                }
            }
        }
    }

    let user = apis::authentication_api::get_current_user(&config)
        .await
        .unwrap();

    match user {
        EitherUserOrTwoFactor::CurrentUser(user) => {
            write_secret_in_directory(jar.clone());
            println!("Logged in as: {}", user.username.unwrap())
        }
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => eprintln!("cookie invalid"),
    }
}

pub async fn check_auth_cookie() {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    if let Some(cookies) = read_secret_in_directory() {
        jar.set_cookies(
            &mut [HeaderValue::from_str(&format!(
                "{}; {}",
                cookies.first().unwrap(),
                cookies.get(1).unwrap()
            ))
            .expect("Invalid cookie string")]
            .iter(),
            &url::Url::from_str("https://api.vrchat.cloud").expect("Invalid URL"),
        );
    }

    let config = apis::configuration::Configuration {
        user_agent: Some(String::from("my-rust-client/1.0.0")),
        client: reqwest::Client::builder()
            .cookie_provider(jar.clone())
            .build()
            .unwrap(),
        ..Default::default()
    };

    match apis::authentication_api::verify_auth_token(&config).await {
        Ok(_) => println!("Auth cookie is valid"),
        Err(e) => eprintln!("Auth cookie is invalid: {}", e),
    }
}
