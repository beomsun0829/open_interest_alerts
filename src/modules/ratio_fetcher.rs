use lazy_static::lazy_static;
use log::{error, debug};
use serde::{de::DeserializeOwned, Deserialize};
use std::sync::Mutex;
use num_format::{Locale, ToFormattedString};

lazy_static! {
    static ref FUNDING_HISTORY_BEFORE_BTC: Mutex<f64> = Mutex::new(0.0);
    static ref FUNDING_HISTORY_BEFORE_USDT: Mutex<f64> = Mutex::new(0.0);
    static ref LONG_HISTORY_BEFORE_GLOBAL: Mutex<f64> = Mutex::new(0.0);
    static ref SHORT_HISTORY_BEFORE_GLOBAL: Mutex<f64> = Mutex::new(0.0);
    static ref LONG_HISTORY_BEFORE_TRADER_POSITION: Mutex<f64> = Mutex::new(0.0);
    static ref SHORT_HISTORY_BEFORE_TRADER_POSITION: Mutex<f64> = Mutex::new(0.0);
    static ref LONG_HISTORY_BEFORE_TRADER_ACCOUNT: Mutex<f64> = Mutex::new(0.0);
    static ref SHORT_HISTORY_BEFORE_TRADER_ACCOUNT: Mutex<f64> = Mutex::new(0.0);
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct InterestData {
    symbol: String,
    #[serde(rename = "sumOpenInterest")]
    sum_open_interest: String,
    #[serde(rename = "sumOpenInterestValue")]
    sum_open_interest_value: String,
    timestamp: i64,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct LongShortData {
    symbol: String,
    #[serde(rename = "longShortRatio")]
    long_short_ratio: String,
    #[serde(rename = "longAccount")]
    long_account: String,
    #[serde(rename = "shortAccount")]
    short_account: String,
    timestamp:i64,
}

struct Change {
    current: f64,
    diff: f64,
}

fn parse_to_f64(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}

fn calculate_change(current: f64, previous: &Mutex<f64>) -> Option<Change> {
    let mut prev = previous.lock().unwrap();

    if *prev == 0.0 {
        *prev = current; 
        return None;
    }

    let diff = current - *prev;
    *prev = current;

    Some(Change { current, diff })
}

fn fetch_data<T>(url: &str) -> Result<Vec<T>, String>
where T: DeserializeOwned + std::fmt::Debug,
{
    let response = reqwest::blocking::get(url);
    
    let body = match response{
        Ok(resp) => match resp.text(){
            Ok(text) => {
                debug!("Response text: {}", text);
                text
            }
            Err(e) => {
                error!("Error reading response text: {}", e);
                return Err(format!("Error reading response text: {}", e));
            }
        },
        Err(e) =>{
            error!("Error fetching data: {}", e);
            return Err(format!("Error fetching data: {}", e));
        }
    };

    let data: Vec<T> = match serde_json::from_str(&body){
        Ok(json_data) => json_data,
        Err(e) => {
            error!("Error parsing JSON: {}", e);
            return Err(format!("Error parsing JSON: {}", e));
        }
    };
    Ok(data)
}

fn get_last_data<T, F>(data: &[T], key_fn: F) -> Option<&T>
where F: Fn(&T) -> i64,
{
    data.iter().max_by_key(|entry| key_fn(entry))
}

fn funding_history_output(last_funding_history: &InterestData) -> String {
    let sum_open_interest = parse_to_f64(&last_funding_history.sum_open_interest);
    let sum_open_interest_value = parse_to_f64(&last_funding_history.sum_open_interest_value);

    let btc_change = calculate_change(sum_open_interest, &FUNDING_HISTORY_BEFORE_BTC);
    let usdt_change = calculate_change(sum_open_interest_value, &FUNDING_HISTORY_BEFORE_USDT);

    let btc_change_text = match btc_change {
        Some(change) => format!(
            "{} BTC ({:+.2})",
            (change.current.round() as u64).to_formatted_string(&Locale::en),
            (change.diff.round() as i64).to_formatted_string(&Locale::en),
        ),
        None => format!(
            "{} BTC ( - )",
            (sum_open_interest.round() as u64).to_formatted_string(&Locale::en)
        ),
    };

    let usdt_change_text = match usdt_change {
        Some(change) => format!(
            "{} USDT ({}{})",
            (change.current.round() as u64).to_formatted_string(&Locale::en),
            if change.diff >= 0.0 { "+" } else { "-" },
            (change.diff.abs().round() as u64).to_formatted_string(&Locale::en),
        ),
        None => format!(
            "{} USDT ( - )",
            (sum_open_interest_value.round() as u64).to_formatted_string(&Locale::en)
        ),
    };

    let format_text = format!(
        "Open Interest\n{}\n\nNotional Value of Open Interest\n{}\n\n",
        btc_change_text, usdt_change_text
    );

    debug!("format_text : {}", format_text);
    format_text
}

pub fn ratio_fetcher() -> String {
    let mut output_text = String::new();

    const OPEN_INTEREST_HIST_URL: &str = "https://fapi.binance.com/futures/data/openInterestHist?symbol=BTCUSDT&period=5m";
    const GLOBAL_LS_RATIO_URL: &str = "https://fapi.binance.com/futures/data/globalLongShortAccountRatio?symbol=BTCUSDT&period=5m";
    const TOP_TRADER_LS_POSITION_RATIO_URL: &str = "https://fapi.binance.com/futures/data/topLongShortPositionRatio?symbol=BTCUSDT&period=5m";
    const TOP_TRADER_LS_ACCOUNT_RATIO_URL: &str = "https://fapi.binance.com/futures/data/topLongShortAccountRatio?symbol=BTCUSDT&period=5m";

    // Open Interest History
    let open_interest_hist = fetch_data::<InterestData>(OPEN_INTEREST_HIST_URL).expect("Failed to fetch data");
    let last_open_interest = get_last_data(&open_interest_hist, |entry| entry.timestamp)
        .expect("Failed to get last data");

    let funding_history_output = funding_history_output(last_open_interest);
    output_text.push_str(&funding_history_output);

    // Global Long-Short Ratio
    let global_ls_ratio = fetch_data::<LongShortData>(GLOBAL_LS_RATIO_URL).expect("Failed to fetch data");
    let last_global_ls_ratio = get_last_data(&global_ls_ratio, |entry| entry.timestamp)
        .expect("Failed to get last data");

    let global_long = parse_to_f64(&last_global_ls_ratio.long_account) * 100.0;
    let global_short = parse_to_f64(&last_global_ls_ratio.short_account) * 100.0;

    let global_long_change = calculate_change(global_long, &LONG_HISTORY_BEFORE_GLOBAL);
    let global_short_change = calculate_change(global_short, &SHORT_HISTORY_BEFORE_GLOBAL);

    let global_long_text = match global_long_change {
        Some(change) => format!("{:.2}% ({:+.2})", change.current, change.diff),
        None => format!("{:.2}% ( - )", global_long),
    };

    let global_short_text = match global_short_change {
        Some(change) => format!("{:.2}% ({:+.2})", change.current, change.diff),
        None => format!("{:.2}% ( - )", global_short),
    };

    output_text.push_str(&format!(
        "Global Long-Short Ratio\nLong Account: {}\nShort Account: {}\n\n",
        global_long_text, global_short_text
    ));

    // Top Trader Long-Short Position Ratio
    let top_trader_ls_position_ratio = fetch_data::<LongShortData>(TOP_TRADER_LS_POSITION_RATIO_URL).expect("Failed to fetch data");
    let last_top_trader_ls_position_ratio = get_last_data(&top_trader_ls_position_ratio, |entry| entry.timestamp)
        .expect("Failed to get last data");

    let trader_position_long = parse_to_f64(&last_top_trader_ls_position_ratio.long_account) * 100.0;
    let trader_position_short = parse_to_f64(&last_top_trader_ls_position_ratio.short_account) * 100.0;

    let trader_position_long_change = calculate_change(trader_position_long, &LONG_HISTORY_BEFORE_TRADER_POSITION);
    let trader_position_short_change = calculate_change(trader_position_short, &SHORT_HISTORY_BEFORE_TRADER_POSITION);

    let trader_position_long_text = match trader_position_long_change {
        Some(change) => format!("{:.2}% ({:+.2})", change.current, change.diff),
        None => format!("{:.2}% ( - )", trader_position_long),
    };

    let trader_position_short_text = match trader_position_short_change {
        Some(change) => format!("{:.2}% ({:+.2})", change.current, change.diff),
        None => format!("{:.2}% ( - )", trader_position_short),
    };

    output_text.push_str(&format!(
        "Top Trader Long-Short Position Ratio\nLong Position: {}\nShort Position: {}\n\n",
        trader_position_long_text, trader_position_short_text
    ));

    // Top Trader Long-Short Account Ratio
    let top_trader_ls_account_ratio = fetch_data::<LongShortData>(TOP_TRADER_LS_ACCOUNT_RATIO_URL).expect("Failed to fetch data");
    let last_top_trader_ls_account_ratio = get_last_data(&top_trader_ls_account_ratio, |entry| entry.timestamp)
        .expect("Failed to get last data");

    let trader_account_long = parse_to_f64(&last_top_trader_ls_account_ratio.long_account) * 100.0;
    let trader_account_short = parse_to_f64(&last_top_trader_ls_account_ratio.short_account) * 100.0;

    let trader_account_long_change = calculate_change(trader_account_long, &LONG_HISTORY_BEFORE_TRADER_ACCOUNT);
    let trader_account_short_change = calculate_change(trader_account_short, &SHORT_HISTORY_BEFORE_TRADER_ACCOUNT);

    let trader_account_long_text = match trader_account_long_change {
        Some(change) => format!("{:.2}% ({:+.2})", change.current, change.diff),
        None => format!("{:.2}% ( - )", trader_account_long),
    };

    let trader_account_short_text = match trader_account_short_change {
        Some(change) => format!("{:.2}% ({:+.2})", change.current, change.diff),
        None => format!("{:.2}% ( - )", trader_account_short),
    };

    output_text.push_str(&format!(
        "Top Trader Long-Short Account Ratio\nLong Account: {}\nShort Account: {}\n\n",
        trader_account_long_text, trader_account_short_text
    ));

    output_text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger;

    #[test]
    fn test_fetch_data(){
        logger::init_logger(true);
        let url = "https://fapi.binance.com/futures/data/openInterestHist?symbol=BTCUSDT&period=5m";
        let res = fetch_data::<InterestData>(url);
        println!("Open Interest Hist Test Result: {:?}\n\n", res);

        let url = "https://fapi.binance.com/futures/data/globalLongShortAccountRatio?symbol=BTCUSDT&period=5m";
        let res = fetch_data::<LongShortData>(url);
        println!("Global Long Short Account Ratio Test Result: {:?}", res);
    }

    #[test]
    fn test_get_last_data(){
        logger::init_logger(true);
        let url = "https://fapi.binance.com/futures/data/openInterestHist?symbol=BTCUSDT&period=5m";
        let tester = fetch_data::<InterestData>(url).expect("Failed to fetch data");
        let key_fn = |entry: &InterestData| entry.timestamp;
        let res = get_last_data(&tester, key_fn);
        println!("Get Last Data Test Result: {:?}", res);
    }

    #[test]
    fn text_funding_history_output(){
        logger::init_logger(true);
        let tester = InterestData {
            symbol: "BTCUSDT".to_string(),
            sum_open_interest: "88637.56900000".to_string(),
            sum_open_interest_value: "8688981341.44580000".to_string(),
            timestamp: 1732442700000,
        };
        let res = funding_history_output(&tester);
        println!("Test Res : {:?}", res);
    }
    
    #[test]
    fn test_ratio_fetcher() {
        logger::init_logger(true);
        let res = ratio_fetcher();
        println!("Test Result: {:?}", res);
    }
}
