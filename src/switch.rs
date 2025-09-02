use vrchatapi::apis::{self, configuration::Configuration};

pub async fn switch_avatar(configuration: Configuration, avatar_id: &str) {
    println!("Switching to avatar ID: {}", avatar_id);

    let result = apis::avatars_api::select_avatar(&configuration, avatar_id).await;

    match result {
        Ok(user) => println!("Successfully switched to avatar: {}", user.current_avatar),
        Err(e) => eprintln!("Failed to switch avatar: {}", e),
    }
}
