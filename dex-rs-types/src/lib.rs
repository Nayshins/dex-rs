use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

/// Wrapper helpers – panic on NaN only during construction.
pub type Price = NotNan<f64>;
pub type Qty = NotNan<f64>;
pub type FundingRate = NotNan<f64>;

#[inline]
pub fn price(v: f64) -> Price {
    NotNan::new(v).expect("NaN price")
}
#[inline]
pub fn qty(v: f64) -> Qty {
    NotNan::new(v).expect("NaN qty")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trade {
    pub id: String,
    pub ts: u64, // unix ms
    pub side: Side,
    pub price: Price,
    pub qty: Qty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBookLevel {
    pub price: Price,
    pub qty: Qty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBook {
    pub coin: String,
    pub ts: u64,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

/* -------- account-trading prereqs -------- */
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tif {
    Ioc,
    Gtc,
    Alo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderReq {
    pub coin: String,
    pub is_buy: bool,
    pub px: Price,
    pub qty: Qty,
    pub tif: Tif,
    pub reduce_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderId(pub String);

/* ---------- float ⇆ decimal feature gate ---------- */
#[cfg(feature = "decimal")]
mod decimal_aliases {
    pub use rust_decimal::Decimal as Price;
    pub use rust_decimal::Decimal as Qty;
    pub use rust_decimal::Decimal as FundingRate;
}

#[cfg(test)]
mod tests {
    use super::*;
    use price as p; // alias

    #[test]
    fn serde_trade_roundtrip() {
        let t = Trade {
            id: "abc".into(),
            ts: 1,
            side: Side::Buy,
            price: p(65000.0),
            qty: qty(0.001),
        };
        let j = serde_json::to_string(&t).unwrap();
        let back: Trade = serde_json::from_str(&j).unwrap();
        assert_eq!(t, back);
    }
}
