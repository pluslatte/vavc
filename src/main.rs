mod auth;
mod db;
mod fetch;
mod secret;
mod switch;

use clap::ArgGroup;
use clap::{Parser, Subcommand, command};
use std::io::{self, Write};

use crate::auth::check_auth_cookie;
use crate::auth::get_new_auth_cookie;
use crate::auth::make_configuration_with_cookies;
use crate::db::{create_alias_db, get_all_avatars};
use crate::fetch::fetch_avatars;
use crate::switch::switch_avatar;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum AliasCommands {
    Set {
        #[arg(short, long, help = "Alias name")]
        alias: String,

        #[arg(short, long, help = "Avatar ID to associate with the alias")]
        id: String,
    },

    Delete {
        #[arg(short, long, help = "Alias name")]
        alias: String,
    },

    List {},
}

#[derive(Debug, Subcommand)]
enum Commands {
    Alias {
        #[command(subcommand)]
        command: AliasCommands,
    },

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

    #[command(group(ArgGroup::new("switch_method").required(true).args(["id", "query", "alias"])), about = "Change avatar")]
    Switch {
        #[arg(short, long, help = "Avatar ID to switch to")]
        id: Option<String>,

        #[arg(short, long, help = "Local database search query to find avatar")]
        query: Option<String>,

        #[arg(short, long, help = "Avatar name alias to switch to")]
        alias: Option<String>,
    },

    #[command(about = "Search for avatars")]
    Search {
        #[arg(short, long, help = "Search query")]
        query: String,
    },

    #[command(about = "Show all avatars in local database")]
    List {},
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Alias { command } => match command {
            AliasCommands::Set {
                alias,
                id: avatar_id,
            } => {
                if let Err(e) = create_alias_db() {
                    eprintln!("Error opening/creating alias database: {}", e);
                    std::process::exit(1);
                }

                if let Err(e) = db::register_alias(&alias, &avatar_id) {
                    eprintln!("Error registering alias: {}", e);
                    return;
                }
            }

            AliasCommands::Delete { alias } => {
                if let Err(e) = create_alias_db() {
                    eprintln!("Error opening/creating alias database: {}", e);
                    std::process::exit(1);
                }

                if let Err(e) = db::remove_alias(&alias) {
                    eprintln!("Error removing alias: {}", e);
                    return;
                }
            }

            AliasCommands::List {} => {
                if let Err(e) = create_alias_db() {
                    eprintln!("Error opening/creating alias database: {}", e);
                    std::process::exit(1);
                }

                match db::get_all_aliases() {
                    Ok(aliases) => {
                        for (name, avatar_id) in &aliases {
                            println!("{}: {}", name, avatar_id);
                        }

                        println!();
                        println!("Total aliases: {}", &aliases.len());
                    }
                    Err(e) => {
                        eprintln!("Error retrieving aliases from database: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },

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

        Commands::Fetch {} => {
            let avatars = fetch_avatars(make_configuration_with_cookies()).await;
            if let Err(e) = db::rebuild_avatar_db(avatars) {
                eprintln!("Error rebuilding avatar database: {}", e);
                std::process::exit(1);
            }

            println!("Avatar database updated successfully.");
        }

        Commands::Switch {
            id: avatar_id,
            query,
            alias,
        } => {
            if let Some(avatar_id) = avatar_id {
                switch_avatar(make_configuration_with_cookies(), &avatar_id).await;
                return;
            }
            if let Some(query) = query {
                match db::get_avatar_first_hit_by_name(&query) {
                    Ok(avatar) => {
                        println!("Found avatar: {} ({})", avatar.name, avatar.id);
                        switch_avatar(make_configuration_with_cookies(), &avatar.id).await;
                    }
                    Err(e) => {
                        eprintln!("Error retrieving avatar for query '{}': {}", query, e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            if let Some(alias) = alias {
                match db::get_avatar_id_by_alias(&alias) {
                    Ok(avatar_id) => {
                        println!(
                            "Resolved avatar alias in local database: {} ({})",
                            &alias, avatar_id
                        );
                        switch_avatar(make_configuration_with_cookies(), &avatar_id).await;
                        return;
                    }
                    Err(e) => {
                        eprintln!("Error retrieving avatar ID for alias '{}': {}", alias, e);
                        std::process::exit(1);
                    }
                }
            }

            eprintln!("Any of --id, --alias or --query must be provided for switching avatars.");
            std::process::exit(1);
        }

        Commands::Search { query } => match db::get_avatars_by_name(&query) {
            Ok(avatars) => {
                for avatar in &avatars {
                    println!("{}: {}", avatar.name, avatar.id);
                }

                println!();
                println!("Total avatars found: {}", &avatars.len());
            }
            Err(e) => {
                eprintln!("Error retrieving avatars from database: {}", e);
                std::process::exit(1);
            }
        },

        Commands::List {} => {
            if let Ok(avatars) = get_all_avatars() {
                for avatar in &avatars {
                    println!("{}: {}", avatar.name, avatar.id);
                }

                println!();
                println!("Total avatars in database: {}", &avatars.len());
            } else {
                eprintln!("Error retrieving avatars from database.");
                std::process::exit(1);
            }
        }
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
