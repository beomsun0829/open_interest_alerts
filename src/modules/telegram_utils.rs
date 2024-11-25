use std::env;
use log::{info, error};
use dotenv::dotenv;

pub fn send_telegram_message(message: &str) {
    dotenv().ok();

    let api_token = env::var("API_TOKEN").expect("API_TOKEN is not set");
    let chat_id = env::var("CHAT_ID").expect("CHAT_ID is not set");

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
        api_token, chat_id, message
    );

    match reqwest::blocking::get(&url) {
        Ok(_) => {
            info!("Sent message: {}", message);
        }
        Err(e) => {
            println!("Error sending message: {}", e);
            error!("Error sending message: {}", e);
        }
    }
}
