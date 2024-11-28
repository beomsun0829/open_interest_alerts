use std::time::Duration;
use chrono::{Local, Timelike};
use log::info;
use std::thread::sleep;

pub fn wait_until_next_run() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let now = Local::now();
        let next_minute = ((now.minute() / 5) + 1) * 5;
        let mut next_hour = now.hour();

        let mut next_run = now.with_minute(next_minute % 60).unwrap().with_second(0).unwrap();

        if next_minute >= 60 {
            next_hour = (next_hour + 1) % 24;
            next_run = next_run.with_hour(next_hour).unwrap();
        }

        let wait_seconds = (next_run - now).num_seconds();

        if wait_seconds <= 0 {
            sleep(Duration::from_millis(100));
            continue;
        }

        info!(
            "Waiting {} seconds until next run at {}",
            wait_seconds,
            next_run.format("%Y-%m-%d %H:%M:%S")
        );
        
        sleep(Duration::from_secs(wait_seconds as u64));
        break;
    }
    Ok(())
}
