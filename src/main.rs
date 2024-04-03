use configurations;
use binance::api::*;
use binance::market::*;
use binance::model::*;

fn main() {
    let (conf, opt) = configurations::parse();
    let market: Market = Binance::new(None, None);

    // Order book at default depth
    match market.get_depth(conf.symbol) {
        Ok(answer) => println!("{:?}", answer),
        Err(e) => println!("Error: {}", e),
    }
}
