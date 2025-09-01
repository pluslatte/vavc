use clap::{Parser, Subcommand, command};
use std::{
    io::{self, Write},
    ptr::read,
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
    #[command(about = "Get a new auth cookie")]
    Auth {
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
        Commands::Auth { username, password } => handler_auth(username, password).await,
        Commands::Switch { id } => handler_switch(id),
        Commands::Search { query } => handler_search(query),
        Commands::Show { id } => handler_show(id),
    }
}

async fn handler_auth(username: Option<String>, password: Option<String>) {
    let username = match username {
        Some(name) => name,
        None => read_user_input("Enter your username: "),
    };
    let password = match password {
        Some(pass) => pass,
        None => read_user_input("Enter your password: "),
    };

    let config = apis::configuration::Configuration {
        basic_auth: Some((username, Some(password))),
        user_agent: Some(String::from("my-rust-client/1.0.0")),
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
            println!("Logged in as: {}", user.username.unwrap())
        }
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => eprintln!("cookie invalid"),
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
