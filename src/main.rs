use std::io::{self, Write};
use vrchatapi::{
    apis,
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode},
};

#[tokio::main]
async fn main() {
    let username = std::env::args().nth(1).expect("Username not provided");
    let password = std::env::args().nth(2).expect("Password not provided");

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
