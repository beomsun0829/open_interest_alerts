use simplelog::*;
use std::fs::File;

pub fn init_logger(test_env: bool) {
    if test_env {
        // 테스트 환경에서는 콘솔에만 출력
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
        // 일반 실행 환경에서는 파일 및 콘솔 출력
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
