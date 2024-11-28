mod modules;
mod logger;

use modules::ratio_fetcher;
use modules::scheduler;
use modules::telegram_utils;

use log::{info, error};

fn main() {
    const USE_TELEGRAM: bool = true;

    logger::init_logger(false);
    info!("Main loop started");
    info!("USE_TELEGRAM: {}", USE_TELEGRAM);

    loop {
        match scheduler::wait_until_next_run() {
            Ok(_) => {
                let the_message = ratio_fetcher::ratio_fetcher();
                info!("Generated message: {}", the_message);
                if USE_TELEGRAM && !the_message.is_empty(){
                    telegram_utils::send_telegram_message(&the_message);
                }
            }
            Err(e) => {
                error!("Error in main loop: {}", e);
            }
        }
    }
}
