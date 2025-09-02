use std::time::Duration;
use tokio::time::sleep;
use vrchatapi::apis;
use vrchatapi::apis::configuration::Configuration;
use vrchatapi::models::Avatar;

pub async fn fetch_avatars(config: Configuration) -> Vec<Avatar> {
    println!("Fetching avatars...");

    let mut out = Vec::<Avatar>::new();
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
                out.push(avatar.clone());
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

    out
}
