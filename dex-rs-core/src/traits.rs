use crate::DexError;
use async_trait::async_trait;
use dex_rs_types::*;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Position {
    pub coin: String,
    pub size: f64,
    pub entry_px: Option<f64>,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamKind {
    Trades,
    Bbo,
    L2Book,
    Orders,
    Fills,
}

#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub status: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct FillEvent {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub oid: u64,
    pub tid: u64,
    pub time: u64,
    pub fee: String,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Trade(Trade),
    Bbo {
        coin: String,
        bid_px: f64,
        ask_px: f64,
    },
    L2(OrderBook),
    Order(OrderEvent),
    Fill(FillEvent),
}

#[async_trait]
pub trait PerpDex: Send + Sync {
    /* ---------- public market data ---------- */
    async fn trades(&self, coin: &str, limit: usize) -> Result<Vec<Trade>, DexError>;
    async fn orderbook(&self, coin: &str, depth: usize) -> Result<OrderBook, DexError>;
    
    /// Get all mid prices for all assets
    async fn all_mids(&self) -> Result<AllMids, DexError>;
    
    /// Get perpetual market metadata
    async fn meta(&self) -> Result<UniverseMeta, DexError>;
    
    /// Get perpetual market metadata with asset contexts
    async fn meta_and_asset_ctxs(&self) -> Result<MetaAndAssetCtxs, DexError>;
    
    /// Get funding rate history for a coin
    async fn funding_history(&self, coin: &str, start_time: u64, end_time: Option<u64>) -> Result<Vec<FundingHistory>, DexError>;

    /* ---------- account ---------- */
    async fn place_order(&self, req: OrderReq) -> Result<OrderId, DexError>;
    async fn cancel(&self, id: OrderId) -> Result<(), DexError>;
    async fn positions(&self) -> Result<Vec<Position>, DexError>;
    
    /// Get user's perpetual trading state (requires authentication)
    async fn user_state(&self) -> Result<UserState, DexError>;
    
    /// Get user's open orders (requires authentication)
    async fn open_orders(&self) -> Result<Vec<OpenOrder>, DexError>;
    
    /// Get user's fill history (requires authentication)
    async fn user_fills(&self) -> Result<Vec<UserFill>, DexError>;
    
    /// Get user's fill history within time range (requires authentication)
    async fn user_fills_by_time(&self, start_time: u64, end_time: Option<u64>) -> Result<Vec<UserFill>, DexError>;

    /* ---------- streaming ---------- */
    async fn subscribe(
        &self,
        kind: StreamKind,
        coin: Option<&str>,
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> Result<(), DexError>;
}
