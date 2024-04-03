use configurations;
use configurations::{Config, Opt};
use binance::api::*;
use binance::market::*;
use binance::model::*;
use binance::account::*;
use binance::config::Config as BnConfig;
use atomic_interval::AtomicIntervalLight;
use std::time::Duration;
use rust_decimal::prelude::*;

fn main() {
    let (conf, opt) = configurations::parse();
    // println!("conf: {:?}", conf);
    // println!("opt: {:?}", opt);

    let mut bnconfig = BnConfig::default();
    // bnconfig.rest_api_endpoint = "http://127.0.0.1:7788".into();
    let market: Market = Binance::new(None, None);
    let account: Account = Binance::new_with_config(Some(conf.api_key.clone()), Some(conf.api_sec.clone()), &bnconfig);

    println!("cancel all order for {}", &conf.symbol);
    match account.cancel_all_open_orders(&conf.symbol[..]) {
        Ok(answer) => println!("cancel order success! {:?}", answer),
        Err(e) => println!("Error: {:?}", e),
    };

    orderloop(&market, &account, &conf, &opt);

    println!("cancel all order for {}", &conf.symbol);
    match account.cancel_all_open_orders(&conf.symbol[..]) {
        Ok(answer) => println!("cancel order success! {:?}", answer),
        Err(e) => println!("Error: {:?}", e),
    };
}

fn orderloop(market: &Market, account: &Account, conf: &Config, _opt:&Opt){

    // interval
    let period = Duration::from_secs(conf.interval);
    let atomic_interval = AtomicIntervalLight::new(period);

    let mut order_id = None::<u64>;
    let mut open_counter = 0;
    let mut cancel_counter = 0;
    let max_order_n = conf.max_order_count; // 下单n次后退出
    let mut ask_avg_10: f64 = 0.0; // 10档均价
    let mut bid_avg_10: f64 = 0.0; // 10档均价
    loop{
        if cancel_counter >= max_order_n {
            break;
        }
        if atomic_interval.is_ticked() {
            println!("open: {} cancel: {}", open_counter, cancel_counter);
            match order_id{
                Some(oid) => {
                    if cancel_counter >= max_order_n {
                        break;
                    }
                    match account.cancel_order(&conf.symbol, oid) {
                        Ok(answer) => {
                            println!("{:?}", answer);
                            order_id = None;
                            cancel_counter+=1;
                        }
                        Err(e) => {
                            println!("cancel_order Error: {:?}", e);
                        }
                    }
                },
                None => {
                    if open_counter >= max_order_n {
                        continue;
                    }
                    match market.get_depth(&conf.symbol) {
                        Ok(depth) => {
                            ask_avg_10 = depth.asks.iter().take(10).map(|a|a.price).sum();
                            ask_avg_10 /= 10.0;
                            bid_avg_10 = depth.bids.iter().take(10).map(|a|a.price).sum();
                            bid_avg_10 /= 10.0;
                            println!("ask_avg:{} bid_avg:{}", ask_avg_10, bid_avg_10);
                        },
                        Err(e) => println!("Error: {}", e),
                    }

                    let sell_price = Decimal::from_f64(ask_avg_10 * 1.2).unwrap();
                    let sell_amount = Decimal::from_f64(conf.order_size_usd as f64).unwrap() / sell_price;
                    let price_decimal_place = 3;
                    let size_decimal_place = 0;
                    // TODO: add fn format_price_size_by_symbol(symbol, price, size)
                    // TODO: add fn calc_size_for_symbol_price(symbol, price, size_usd)
                    let sell_amount_f64 = sell_amount.round_dp(size_decimal_place).to_f64().unwrap();
                    let sell_price_f64 = sell_price.round_dp(price_decimal_place).to_f64().unwrap();
                    match account.limit_sell(&conf.symbol, sell_amount_f64, sell_price_f64) {
                        Ok(order) => {
                            open_counter+=1;
                            println!("{:?}", order);
                            order_id = Some(order.order_id);
                        }
                        Err(e) => {
                            println!("limit sell Error: {:?}", e);
                            break},
                    }
                },
            };
        }
    };
}



