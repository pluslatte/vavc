mod auth;
mod secret;

use clap::{Parser, Subcommand, command};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;
use vrchatapi::apis;
use vrchatapi::apis::configuration::Configuration;

use crate::auth::check_auth_cookie;
use crate::auth::get_new_auth_cookie;
use crate::auth::make_configuration_with_cookies;

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

    #[command(about = "Fetch avatars to local database")]
    Fetch {},

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
        Commands::Fetch {} => handler_fetch(make_configuration_with_cookies()).await,
        Commands::Switch { id } => handler_switch(id),
        Commands::Search { query } => {
            handler_search(make_configuration_with_cookies(), query).await
        }
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

async fn handler_fetch(config: Configuration) {
    println!("Fetching avatars...");

    let mut avatar_count: usize = 0;

    loop {
        let avatars = apis::avatars_api::search_avatars(
            &config,
            Some(false),
            Some(vrchatapi::models::SortOption::Name),
            Some("me"),
            None,
            Some(60),
            None,
            Some(avatar_count.try_into().expect("Negative avatar count wtf")),
            None,
            None,
            Some(vrchatapi::models::ReleaseStatus::All),
            None,
            None,
            None,
        )
        .await;

        if let Ok(avatars) = avatars {
            let got = avatars.len();
            if got == 0 {
                break;
            }

            println!(
                "Fetched {} avatars, total so far: {}",
                got,
                avatar_count + got
            );

            avatar_count += got;
            avatars.iter().for_each(|avatar| {
                println!("{}: {}", avatar.name, avatar.id);
            });

            println!("Sleep for 5 seconds to avoid rate limiting...");
            sleep(Duration::from_secs(5)).await; // To avoid rate limiting
            continue;
        } else {
            eprintln!("Failed to fetch avatars: {}", avatars.err().unwrap());
            break;
        }
    }

    println!(
        "Finished fetching avatars. Total avatars fetched: {}",
        avatar_count
    );
}

async fn handler_search(config: Configuration, query: String) {}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}
