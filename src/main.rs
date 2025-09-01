use clap::{Parser, Subcommand, command};
use reqwest::{
    cookie::{self, CookieStore},
    header::HeaderValue,
};
use std::{
    io::{self, Write},
    str::FromStr,
    sync::Arc,
};
use vrchatapi::{
    apis,
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode},
};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "(RATE LIMIT WARNING) Get a new auth cookie")]
    Auth {
        #[arg(short, long, help = "Check if your saved cookie is valid")]
        check: bool,

        #[arg(short, long, help = "Optional pre-input username")]
        username: Option<String>,

        #[arg(short, long, help = "Optional pre-input password")]
        password: Option<String>,
    },

    #[command(about = "Change avatar")]
    Switch {
        #[arg(short, long, help = "Avatar ID to switch to")]
        id: String,
    },

    #[command(about = "Search for avatars")]
    Search {
        #[arg(short, long, help = "Search query")]
        query: String,
    },

    #[command(about = "Show specifications about an avatar")]
    Show {
        #[arg(short, long, help = "Avatar ID to show")]
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let handler_switch = |id: String| {
        // Placeholder for switch logic
        println!("Switching to avatar ID: {}", id);
    };

    let handler_search = |query: String| {
        // Placeholder for search logic
        println!("Searching for avatars with query: {}", query);
    };

    let handler_show = |id: String| {
        // Placeholder for show logic
        println!("Showing specifications for avatar ID: {}", id);
    };

    match cli.command {
        Commands::Auth {
            username,
            password,
            check,
        } => handler_auth(username, password, check).await,
        Commands::Switch { id } => handler_switch(id),
        Commands::Search { query } => handler_search(query),
        Commands::Show { id } => handler_show(id),
    }
}

async fn handler_auth(username: Option<String>, password: Option<String>, check: bool) {
    if check {
        check_auth_cookie().await;
    } else {
        get_new_auth_cookie(username, password).await;
    }
}

async fn get_new_auth_cookie(username: Option<String>, password: Option<String>) {
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

async fn check_auth_cookie() {
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

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

fn read_secret_in_directory() -> Option<Vec<String>> {
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

fn write_secret_in_directory<C>(cookie_store: Arc<C>)
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
