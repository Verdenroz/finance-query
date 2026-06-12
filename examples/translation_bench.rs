//! Latency benchmark for the offline translation backend.
//!
//! Run: cargo run --profile bench --example translation_bench --features translation-offline
//!
//! Payload approximates a worst-case server endpoint: a batch of news
//! headlines plus multi-sentence business summaries. Each target language
//! is a cold run (the memo cache is keyed per language).

use std::time::Instant;

use finance_query::translation::{self, Lang};

const HEADLINES: [&str; 20] = [
    "Apple shares rise after record quarterly earnings beat analyst expectations.",
    "Federal Reserve signals two more rate cuts before the end of the year.",
    "Nvidia unveils next-generation AI accelerator with double the memory bandwidth.",
    "Oil prices slip as OPEC+ members debate extending production cuts into 2027.",
    "Tesla deliveries fall short of estimates amid intensifying competition in China.",
    "Microsoft expands data center footprint with $40 billion investment in Europe.",
    "Treasury yields climb after stronger-than-expected jobs report.",
    "Amazon announces stock split and accelerates share buyback program.",
    "Gold hits all-time high as investors seek safe havens amid trade tensions.",
    "Semiconductor stocks rally on easing export restrictions to key markets.",
    "JPMorgan raises S&P 500 year-end target citing resilient consumer spending.",
    "Bitcoin tops $150,000 as institutional inflows reach record levels.",
    "Boeing secures largest-ever widebody order from Gulf carriers.",
    "Eurozone inflation cools faster than forecast, pressuring the ECB to act.",
    "Berkshire Hathaway trims Apple stake while boosting energy holdings.",
    "Meta beats revenue estimates on stronger advertising demand.",
    "Japanese yen weakens past key level, prompting intervention warnings.",
    "Retail sales rebound in May as consumers shrug off tariff concerns.",
    "Alphabet announces dividend increase alongside cloud revenue acceleration.",
    "Small-cap stocks outperform as rate cut expectations broaden market rally.",
];

const SUMMARIES: [&str; 2] = [
    "Apple Inc. designs, manufactures, and markets smartphones, personal computers, \
     tablets, wearables, and accessories worldwide. The company offers iPhone, a line \
     of smartphones; Mac, a line of personal computers; iPad, a line of multi-purpose \
     tablets; and wearables, home, and accessories comprising AirPods, Apple TV, \
     Apple Watch, Beats products, and HomePod. It also provides AppleCare support and \
     cloud services; and operates various platforms, including the App Store. In \
     addition, the company offers various services, such as Apple Arcade, a game \
     subscription service; Apple Fitness+, a personalized fitness service; Apple \
     Music, which offers users a curated listening experience; Apple News+, a \
     subscription news and magazine service; Apple Pay, a cashless payment service; \
     Apple TV+, which offers exclusive original content; and Apple Card, a co-branded \
     credit card. The company was incorporated in 1977 and is headquartered in \
     Cupertino, California.",
    "NVIDIA Corporation provides graphics and compute and networking solutions in the \
     United States, Taiwan, China, Hong Kong, and internationally. The Compute & \
     Networking segment comprises Data Center computing platforms and end-to-end \
     networking platforms; NVIDIA DRIVE automated-driving platform and automotive \
     development agreements; Jetson robotics and other embedded platforms; NVIDIA AI \
     Enterprise and other software; and DGX Cloud software and services. The Graphics \
     segment offers GeForce GPUs for gaming and PCs, the GeForce NOW game streaming \
     service and related infrastructure, and solutions for gaming platforms. The \
     company's products are used in gaming, professional visualization, data center, \
     and automotive markets.",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let texts: Vec<String> = HEADLINES
        .iter()
        .chain(SUMMARIES.iter())
        .map(|s| s.to_string())
        .collect();

    let t = Instant::now();
    translation::preload().await?;
    println!("model load: {:?}", t.elapsed());
    println!("payload: {} texts", texts.len());

    for code in ["de", "ja", "es"] {
        let lang = Lang::parse(code)?;
        let t = Instant::now();
        let out = translation::translate_texts(&texts, &lang).await?;
        let elapsed = t.elapsed();
        println!(
            "{code}: {:>8.2?}  ({:.1} texts/s)",
            elapsed,
            texts.len() as f64 / elapsed.as_secs_f64()
        );
        println!("    sample: {}", &out[0]);
    }
    Ok(())
}
