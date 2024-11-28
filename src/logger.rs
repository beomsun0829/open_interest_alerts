use simplelog::*;
use std::fs::File;

pub fn init_logger(test_env: bool) {
    if test_env {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
        ])
        .unwrap();
    }
    
    else {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create("log.txt").unwrap(),
            ),
        ])
        .unwrap();
    }
}
