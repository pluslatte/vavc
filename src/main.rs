mod auth;
mod fetch;
mod secret;
mod switch;

use clap::ArgGroup;
use clap::{Parser, Subcommand, command};
use std::io::{self, Write};
use vrchatapi::apis::configuration::Configuration;

use crate::auth::check_auth_cookie;
use crate::auth::get_new_auth_cookie;
use crate::auth::make_configuration_with_cookies;
use crate::fetch::fetch_avatars;
use crate::switch::switch_avatar;

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

    #[command(about = "Fetch avatars to local database")]
    Fetch {},

    #[command(group(ArgGroup::new("switch_method").required(true).args(["id", "query"])), about = "Change avatar")]
    Switch {
        #[arg(short, long, help = "Avatar ID to switch to")]
        id: Option<String>,

        #[arg(short, long, help = "Local database search query to find avatar")]
        query: Option<String>,
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

    let handler_search = |config: Configuration, query: String| {
        // Placeholder for search logic
        println!("Searching for avatars with query: {}", query);
    };

    let handler_show = |avatar_id: String| {
        // Placeholder for show logic
        println!("Showing specifications for avatar ID: {}", avatar_id);
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

        Commands::Switch {
            id: avatar_id,
            query,
        } => {
            if let Some(avatar_id) = avatar_id {
                switch_avatar(make_configuration_with_cookies(), &avatar_id).await;
                return;
            }
            if let Some(query) = query {
                todo!("Implement switch by search query");
                return;
            }

            eprintln!("Either --id or --query must be provided for switching avatars.");
            std::process::exit(1);
        }

        Commands::Search { query } => handler_search(make_configuration_with_cookies(), query),

        Commands::Show { id: avatar_id } => handler_show(avatar_id),
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
