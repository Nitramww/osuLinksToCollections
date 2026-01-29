use rosu_v2::prelude::*;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};
use tokio::time::{sleep, Duration};
use url::Url;

#[derive(Debug, Clone)]
pub struct BeatmapInfo {
    pub checksum: String,
    pub mapset_id: u32,
}

/// Extract beatmap ID from URL
fn extract_beatmap_id(url_str: &str) -> Option<u32> {
    if let Ok(id) = url_str.parse::<u32>() {
        return Some(id);
    }

    let url = Url::parse(url_str).ok()?;
    
    // Handle /beatmaps/{id} or /b/{id}
    if let Some(mut segments) = url.path_segments() {
        if let Some(first) = segments.next() {
            if first == "beatmaps" || first == "b" {
                if let Some(id_str) = segments.next() {
                    return id_str.parse::<u32>().ok();
                }
            }
        }
    }
    
    if let Some(fragment) = url.fragment() {
        let parts: Vec<&str> = fragment.split('/').collect();
        if parts.len() == 2 {
            // parts[0] is the mode (osu, mania, taiko, fruits)
            // parts[1] is the beatmap id
            return parts[1].parse::<u32>().ok();
        }
    }
    
    None
}

pub async fn fetch_beatmaps(
    client_id: u64,
    client_secret: &str,
) -> Result<Vec<BeatmapInfo>, Box<dyn std::error::Error>> {
    // Build osu client
    let osu = OsuBuilder::new()
        .client_id(client_id)
        .client_secret(client_secret.to_string())
        .ratelimit(1) // 1 requests/sec 
        .build()
        .await?;

    println!("API client initialized");

    // Read links from file
    let file = File::open("links.txt")?;
    let reader = BufReader::new(file);
    
    let urls: Vec<String> = reader
        .lines()
        .filter_map(Result::ok)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#')) // Allow comments with #
        .collect();

    println!("Found {} URLs to process", urls.len());

    // Open output file (overwrite if exists to avoid duplicates)
    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("collection_hashes.txt")?;

    let delay = Duration::from_millis(1000); // 1000ms delay || 1 requests/sec
    let mut results = Vec::new();
    let mut failed_urls = Vec::new();

    for (i, url_str) in urls.iter().enumerate() {
        if let Some(id) = extract_beatmap_id(url_str) {
            match osu.beatmap().map_id(id).await {
                Ok(bm) => {
                    if let Some(checksum) = bm.checksum {
                        let line = format!("{}|{}\n", checksum, bm.mapset_id);
                        output.write_all(line.as_bytes())?;
                        
                        results.push(BeatmapInfo {
                            checksum,
                            mapset_id: bm.mapset_id,
                        });
                        
                        println!(
                            "[{:>3}/{:>3}] + Saved: {} (mapset: {})",
                            i + 1,
                            urls.len(),
                            id,
                            bm.mapset_id
                        );
                    } else {
                        eprintln!("[{:>3}/{:>3}] - Beatmap {} missing checksum", i + 1, urls.len(), id);
                        failed_urls.push((url_str.clone(), format!("Missing checksum")));
                    }
                }
                Err(err) => {
                    eprintln!("[{:>3}/{:>3}] - Failed to fetch beatmap {}: {}", i + 1, urls.len(), id, err);
                    failed_urls.push((url_str.clone(), err.to_string()));
                }
            }
            
            // Rate limiting delay
            sleep(delay).await;
        } else {
            eprintln!("[{:>3}/{:>3}] - Invalid URL: {}", i + 1, urls.len(), url_str);
            failed_urls.push((url_str.clone(), "Invalid URL format".to_string()));
        }
    }

    println!("\n=== Summary ===");
    println!("+ Successfully fetched: {}", results.len());
    println!("- Failed: {}", failed_urls.len());
    
    if !failed_urls.is_empty() {
        println!("\nFailed URLs:");
        for (url, reason) in failed_urls {
            println!("  - {} ({})", url, reason);
        }
    }

    Ok(results)
}