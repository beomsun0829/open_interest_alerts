mod modules;
mod logger;

use modules::ratio_fetcher;
use modules::scheduler;
use modules::telegram_utils;

use log::{info, error};

fn main() {
    logger::init_logger(false);
    println!("Start");
    info!("Main loop started");

    loop {
        match scheduler::wait_until_next_run() {
            Ok(_) => {
                let the_message = ratio_fetcher::fetch_longshort_ratio();
                println!("Message: {}", the_message);
                info!("Generated message: {}", the_message);
                if !the_message.is_empty() {
                    telegram_utils::send_telegram_message(&the_message);
                }
            }
            Err(e) => {
                println!("Error in main loop: {}", e);
                error!("Error in main loop: {}", e);
            }
        }
    }
}
