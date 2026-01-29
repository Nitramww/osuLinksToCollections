# osu! Links to Collection

A Rust application that fetches beatmap data from the osu! API and creates a `collection.db` file for importing into Collection Manager.

## Usage

### 1. Create `links.txt`
```
https://osu.ppy.sh/beatmaps/5070035
https://osu.ppy.sh/beatmapsets/410162#osu/890190
# This is a comment - it will be ignored
https://osu.ppy.sh/b/2533647
1945546
```

### 2. Set your API credentials in main.rs
Edit lines 18-19:
```rust
let client_id: u64 = YOUR_CLIENT_ID;
let client_secret = "YOUR_CLIENT_SECRET";
```

Get credentials at: https://osu.ppy.sh/home/account/edit#oauth

### 3. Run the program
```bash
cargo run --release
```

### 4. Subsequent runs (no API calls!)
Once `collection_hashes.txt` exists, the program uses cached data:
```bash
cargo run
```

To fetch fresh data, delete the cache:
```bash
rm collection_hashes.txt  # Force refresh
cargo run
```

### 5. Output
The program will:
1. Fetch beatmap data from the API
2. Save hashes to `collection_hashes.txt`
3. Create `collection.db` file
4. Display progress and summary

## Example Output

**First run (with API calls):**
```
=== osu! Collection Builder ===

Credentials loaded:
  Client ID: 12345
  Client Secret: abcd...wxyz

+ API client initialized
Found 150 URLs to process
[  1/150] + Saved: 123456 (mapset: 789012)
[  2/150] + Saved: 234567 (mapset: 789012)
[  3/150] - Failed to fetch beatmap 999999: Not found
...
[150/150] + Saved: 345678 (mapset: 123456)

=== Summary ===
+ Successfully fetched: 148
- Failed: 2

Failed URLs:
  - https://osu.ppy.sh/beatmaps/999999 (Not found)
  - https://invalid-url (Invalid URL format)

Loaded 148 beatmap hashes from file
+ Collection saved successfully with 148 beatmaps!
```

**Subsequent runs (using cache):**
```
=== osu! Collection Builder ===

+ Found existing collection_hashes.txt
  Skipping API calls and using cached data
  (Delete this file to fetch fresh data from API)

Loaded 148 beatmap hashes from file
+ Collection saved successfully with 148 beatmaps!
```

## File Structure

- `main.rs` - Entry point, handles collection creation
- `api_data.rs` - API interaction and URL parsing
- `links.txt` - Input file with beatmap URLs (one per line)
- `collection_hashes.txt` - Generated file with hash|mapset_id pairs
- `collection.db` - Final output file

## Dependencies

```toml
[dependencies]
osu-db = "0.1"
rosu-v2 = "0.9"
tokio = { version = "1", features = ["full"] }
url = "2"
```

## Rate Limiting

The code respects osu! API rate limits:
- 1 requests per second (60 per minute)
- 1000ms delay between requests
- Configurable via `ratelimit()` in OsuBuilder

## Notes

- Empty lines and comments (starting with #) in `links.txt` are ignored
- The program overwrites `collection_hashes.txt` on each run to avoid duplicates
- Failed fetches are logged but don't stop the process
- Beatmaps without checksums are skipped with a warning
