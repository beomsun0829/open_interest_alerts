use lazy_static::lazy_static;
use log::{error, debug, info};
use serde::{de::DeserializeOwned, Deserialize};
use std::sync::Mutex;
use num_format::{Locale, ToFormattedString};

lazy_static! {
    static ref FUNDING_HISTORY_BEFORE_BTC: Mutex<f64> = Mutex::new(0.0);
    static ref FUNDING_HISTORY_BEFORE_USDT: Mutex<f64> = Mutex::new(0.0);
    static ref LONG_HISTORY_BEFORE_BTC: Mutex<f64> = Mutex::new(0.0);
    static ref SHORT_HISTORY_BEFORE_BTC: Mutex<f64> = Mutex::new(0.0);
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

fn funding_history_output(last_funding_history: InterestData) -> String {
    let sum_open_interest = parse_to_f64(&last_funding_history.sum_open_interest);
    let sum_open_interest_value = parse_to_f64(&last_funding_history.sum_open_interest_value);

    let btc_change = calculate_change(sum_open_interest, &FUNDING_HISTORY_BEFORE_BTC);
    let usdt_change = calculate_change(sum_open_interest_value, &FUNDING_HISTORY_BEFORE_USDT);

    let btc_change_text = match btc_change {
        Some(change) => format!(
            "{} BTC ({:+.3})",
            change.current
                .round()
                .to_string()
                .parse::<u64>()
                .unwrap_or(0)
                .to_formatted_string(&Locale::en),
            change.diff
        ),
        None => format!(
            "{} BTC ( - )",
            sum_open_interest
                .round()
                .to_string()
                .parse::<u64>()
                .unwrap_or(0)
                .to_formatted_string(&Locale::en)
        ),
    };

    let usdt_change_text = match usdt_change {
        Some(change) => format!(
            "{} USDT ({:+.0})",
            change.current
                .round()
                .to_string()
                .parse::<u64>()
                .unwrap_or(0)
                .to_formatted_string(&Locale::en),
            change.diff
        ),
        None => format!(
            "{} USDT ( - )",
            sum_open_interest_value
                .round()
                .to_string()
                .parse::<u64>()
                .unwrap_or(0)
                .to_formatted_string(&Locale::en)
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
    


    output_text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger;

    #[test]
    fn test_fetch_data(){
        logger::init_logger(true);
        let url = "https://www.binance.com/futures/data/openInterestHist?symbol=BTCUSDT&period=5m";
        let res = fetch_data::<InterestData>(url);
        println!("Open Interest Hist Test Result: {:?}\n\n", res);

        let url = "https://www.binance.com/futures/data/globalLongShortAccountRatio?symbol=BTCUSDT&period=5m";
        let res = fetch_data::<LongShortData>(url);
        println!("Global Long Short Account Ratio Test Result: {:?}", res);
    }

    #[test]
    fn test_get_last_data(){
        logger::init_logger(true);
        let url = "https://www.binance.com/futures/data/openInterestHist?symbol=BTCUSDT&period=5m";
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
        let res = funding_history_output(tester);
        println!("Test Res : {:?}", res);
    }
    
    #[test]
    fn test_ratio_fetcher() {
        logger::init_logger(true);
        let res = ratio_fetcher();
        println!("Test Result: {:?}", res);
    }
}
