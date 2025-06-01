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

/* -------- extended API types -------- */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetPosition {
    pub coin: String,
    pub hold: String,
    pub szi: String,
    pub leverage: Option<FundingRate>,
    pub entry_px: Option<Price>,
    pub position_value: String,
    pub unrealized_pnl: String,
    pub return_on_equity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarginSummary {
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossMarginSummary {
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WithdrawalsUsed {
    pub used: String,
    pub limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserState {
    pub asset_positions: Vec<AssetPosition>,
    pub cross_margin_summary: CrossMarginSummary,
    pub cross_maintenance_margin_used: String,
    pub withdrawals_used: Vec<WithdrawalsUsed>,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenOrder {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub timestamp: u64,
    pub orig_sz: String,
    pub cloid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFill {
    pub coin: String,
    pub px: String,
    pub sz: String,
    pub side: String,
    pub time: u64,
    pub start_position: String,
    pub dir: String,
    pub closed_pnl: String,
    pub hash: String,
    pub oid: u64,
    pub crossed: bool,
    pub fee: String,
    pub tid: u64,
    pub liquidation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FundingHistory {
    pub coin: String,
    pub funding_rate: String,
    pub premium: String,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetMeta {
    pub name: String,
    pub sz_decimals: u32,
    pub max_leverage: u32,
    pub only_isolated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniverseMeta {
    pub assets: Vec<AssetMeta>,
    pub universe: Vec<UniverseItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniverseItem {
    pub name: String,
    pub index: u32,
    pub tokens: Vec<u32>,
    pub is_canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetCtx {
    pub funding: String,
    pub open_interest: String,
    pub prev_day_px: String,
    pub day_ntl_vlm: String,
    pub premium: String,
    pub oracle_px: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub impact_px: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaAndAssetCtxs {
    pub meta: UniverseMeta,
    pub asset_ctxs: Vec<AssetCtx>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotAssetMeta {
    pub name: String,
    pub sz_decimals: u32,
    pub wei_decimals: u32,
    pub index: u32,
    pub token_id: String,
    pub is_canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotMeta {
    pub tokens: Vec<SpotAssetMeta>,
    pub universe: Vec<SpotUniverseItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotUniverseItem {
    pub tokens: Vec<u32>,
    pub name: String,
    pub index: u32,
    pub is_canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotAssetCtx {
    pub day_ntl_vlm: String,
    pub prev_day_px: String,
    pub mark_px: Option<String>,
    pub mid_px: Option<String>,
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpotMetaAndAssetCtxs {
    pub meta: SpotMeta,
    pub asset_ctxs: Vec<SpotAssetCtx>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllMids {
    pub mids: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFees {
    pub total_fees: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    pub time: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandleSnapshot(pub Vec<Candle>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderStatus {
    pub order: Option<OpenOrder>,
    pub status: String,
    pub status_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFunding {
    pub delta: Vec<UserFundingDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFundingDelta {
    pub coin: String,
    pub funding_rate: String,
    pub szi: String,
    pub usdc: String,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DelegatorSummary {
    pub total_delegated: String,
    pub total_rewards: String,
    pub total_penalties: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Delegation {
    pub validator: String,
    pub amount: String,
    pub rewards: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DelegatorRewards {
    pub rewards: Vec<DelegationReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DelegationReward {
    pub validator: String,
    pub rewards: String,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferralState {
    pub code: String,
    pub referred_by: Option<String>,
    pub total_referrals: u32,
    pub total_volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubAccount {
    pub sub_account_user: String,
    pub name: String,
}

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
