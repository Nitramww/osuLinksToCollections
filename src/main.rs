use osu_db::collection::{Collection, CollectionList};
use std::fs::File;
use std::io::{BufRead, BufReader};

mod api_data;
use api_data::BeatmapInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== osu! Collection Builder ===\n");

    // Check if we can skip API calls by using cached data
    let use_cache = std::path::Path::new("collection_hashes.txt").exists();

    let collection_name = "lowkirkawesomlysauce".to_string(); // Change this to change collection name when imported

    if use_cache {
        println!("+ Found existing collection_hashes.txt");
        println!("  Skipping API calls and using cached data");
        println!("  (Delete this file to fetch fresh data from API)\n");
    } else {
        // API credentials - replace with your own
        let client_id: u64 = 12345;
        let client_secret = "your_super_secret_key_here";

        println!("Fetching beatmaps from API...");
        match api_data::fetch_beatmaps(client_id, &client_secret).await {
            Ok(beatmaps) => {
                println!("+ Fetched {} beatmaps\n", beatmaps.len());
                
                if beatmaps.is_empty() {
                    eprintln!("???  Warning: No beatmaps were successfully fetched!");
                    eprintln!("   Check that links.txt exists and contains valid osu! beatmap URLs\n");
                }
            }
            Err(e) => {
                eprintln!("- Failed to fetch beatmaps: {}\n", e);
                
                if e.to_string().contains("invalid_client") || e.to_string().contains("401") {
                    eprintln!("This looks like an authentication error. Common causes:");
                    eprintln!("  • Client ID or Client Secret is incorrect");
                    eprintln!("  • Extra spaces in credentials");
                    eprintln!("  • OAuth application was deleted or disabled");
                    eprintln!("\n?? Get your credentials at:");
                    eprintln!("   https://osu.ppy.sh/home/account/edit#oauth");
                    eprintln!("\n?? Then update main.rs lines 18-19:");
                    eprintln!("   let client_id: u64 = YOUR_CLIENT_ID;");
                    eprintln!("   let client_secret = \"YOUR_CLIENT_SECRET\";\n");
                }
                
                return Err(e);
            }
        }
    }

    // Read the saved collection_hashes.txt
    let file = match File::open("collection_hashes.txt") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("-  Failed to open collection_hashes.txt: {}", e);
            eprintln!("   This file should have been created during the API fetch.");
            return Err(e.into());
        }
    };
    
    let reader = BufReader::new(file);
    
    let beatmap_hashes: Vec<Option<String>> = reader
        .lines()
        .filter_map(Result::ok)
        .filter_map(|line| {
            line.split('|')
                .next()
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .map(Some)
        .collect();

    println!("Loaded {} beatmap hashes from file", beatmap_hashes.len());

    if beatmap_hashes.is_empty() {
        eprintln!("!!!  Warning: No valid hashes found in collection_hashes.txt");
        return Err("No beatmaps to add to collection".into());
    }

    // Create a single collection
    let collection = Collection {
        name: Some(collection_name),
        beatmap_hashes,
    };

    // Wrap it in a CollectionList
    let collection_list = CollectionList {
        version: 20220906,
        collections: vec![collection],
    };

    // Save to collection.db
    match collection_list.to_file("collection.db") {
        Ok(_) => {
            println!(
                "\n+ Collection saved successfully with {} beatmaps!",
                collection_list.collections[0].beatmap_hashes.len()
            );
            println!("\n   Output files created:");
            println!("   • collection.db");
            println!("   • collection_hashes.txt (cached data)");
            println!("\n   To import into osu!:");
            println!("   1. Close osu! completely");
            println!("   2. Add collection.db to osu! collections using Collection Manager");
            println!("   3. Downloading missing beatmaps using Batch Beatmap Downloader");
        }
        Err(e) => {
            eprintln!("- Failed to save collection.db: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}