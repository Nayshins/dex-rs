use dex_rs::prelude::*;

#[tokio::main]
async fn main() -> DexResult<()> {
    env_logger::init();

    println!("🔗 Connecting to Hyperliquid testnet...");
    let hl = Hyperliquid::builder().testnet().connect().await?;

    println!("🌍 Fetching market metadata...");
    let meta = hl.meta().await?;

    println!("📊 Fetching asset contexts...");
    let meta_and_contexts = hl.meta_and_asset_ctxs().await?;

    println!("💰 Fetching all mid prices...");
    let all_mids = hl.all_mids().await?;

    println!("\n🏪 Hyperliquid Market Overview");
    println!("{:=<80}", "");

    println!("\n📈 Available Assets ({} total):", meta.assets.len());
    println!("{:-<80}", "");
    println!(
        "{:<10} {:<15} {:<12} {:<15} {:<15}",
        "Symbol", "Mid Price", "Max Lev", "Oracle Price", "Mark Price"
    );
    println!("{:-<80}", "");

    for (i, asset) in meta.assets.iter().enumerate().take(20) {
        // Show first 20 assets
        let mid_price = all_mids
            .mids
            .get(&asset.name)
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);

        let asset_ctx = meta_and_contexts.asset_ctxs.get(i);

        let (oracle_price, mark_price) = if let Some(ctx) = asset_ctx {
            let oracle: f64 = ctx.oracle_px.parse().unwrap_or(0.0);
            let mark: f64 = ctx.mark_px.parse().unwrap_or(0.0);
            (oracle, mark)
        } else {
            (0.0, 0.0)
        };

        println!(
            "{:<10} ${:<14.2} {:<12}x ${:<14.2} ${:<14.2}",
            asset.name, mid_price, asset.max_leverage, oracle_price, mark_price
        );
    }

    if meta.assets.len() > 20 {
        println!("... and {} more assets", meta.assets.len() - 20);
    }

    println!("\n🎯 Market Statistics:");
    println!("{:-<50}", "");

    // Find most active assets by checking which have the highest volumes
    let mut volume_assets: Vec<_> = meta_and_contexts
        .asset_ctxs
        .iter()
        .enumerate()
        .filter_map(|(i, ctx)| {
            if let Some(asset) = meta.assets.get(i) {
                let volume: f64 = ctx.day_ntl_vlm.parse().unwrap_or(0.0);
                if volume > 0.0 {
                    Some((asset.name.clone(), volume))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    volume_assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("📊 Top 5 by Volume (24h):");
    for (i, (name, volume)) in volume_assets.iter().take(5).enumerate() {
        println!("  {}. {}: ${:.0}M", i + 1, name, volume / 1_000_000.0);
    }

    // Show universe information
    println!("\n🌐 Universe Information:");
    println!("Total assets: {}", meta.universe.len());
    let canonical_count = meta.universe.iter().filter(|u| u.is_canonical).count();
    println!("Canonical assets: {}", canonical_count);

    // Show some interesting assets
    println!("\n🔍 Special Assets:");
    for asset in &meta.assets {
        if asset.only_isolated {
            println!(
                "  🔒 {} (Isolated only, max {}x leverage)",
                asset.name, asset.max_leverage
            );
        }
    }

    Ok(())
}
