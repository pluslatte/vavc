mod auth;
mod fetch;
mod secret;

use clap::{Parser, Subcommand, command};
use std::io::{self, Write};
use vrchatapi::apis::configuration::Configuration;

use crate::auth::check_auth_cookie;
use crate::auth::get_new_auth_cookie;
use crate::auth::make_configuration_with_cookies;
use crate::fetch::fetch_avatars;

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

    let handler_search = |config: Configuration, query: String| {
        // Placeholder for search logic
        println!("Searching for avatars with query: {}", query);
    };

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
        } => {
            if check {
                check_auth_cookie().await
            } else {
                get_new_auth_cookie(username, password).await
            };
        }
        Commands::Fetch {} => fetch_avatars(make_configuration_with_cookies()).await,
        Commands::Switch { id } => handler_switch(id),
        Commands::Search { query } => handler_search(make_configuration_with_cookies(), query),
        Commands::Show { id } => handler_show(id),
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
