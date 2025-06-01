use dex_rs::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> DexResult<()> {
    let hl = Hyperliquid::builder().testnet().connect().await?;
    println!("Last trade: {:?}", hl.trades("BTC", 1).await?.pop());

    let (tx, mut rx) = mpsc::unbounded_channel();
    hl.subscribe(StreamKind::Bbo, Some("BTC"), tx).await?;

    while let Some(ev) = rx.recv().await {
        if let StreamEvent::Bbo { bid_px, ask_px, .. } = ev {
            println!("Bid {bid_px}   Ask {ask_px}");
            break; // demo – exit after first tick
        }
    }
    Ok(())
}