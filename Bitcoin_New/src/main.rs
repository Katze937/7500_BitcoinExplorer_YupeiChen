use std::{thread, time};
use mysql::*;
use mysql::prelude::*;
use reqwest::Client;
use serde_json::json;
use chrono::NaiveDateTime; // Keep only what you need


fn clear_table(pool: &Pool) -> mysql::Result<()> {
    let mut conn = pool.get_conn()?;
    conn.query_drop("TRUNCATE TABLE blocks")?; // Use TRUNCATE to clear and reset
    println!("Table 'blocks' cleared and AUTO_INCREMENT reset.");
    Ok(())
}

#[tokio::main]
async fn main() {
    // Bitcoin RPC credentials
    let rpc_user = "myusername"; // Replace with your Bitcoin Core RPC username
    let rpc_password = "mypassword"; // Replace with your Bitcoin Core RPC password
    let url = "http://127.0.0.1:8332/";

    // Connect to MySQL database
    let mysql_url = "mysql://root:Aisa937!@127.0.0.1:3306/bitcoin_data"; // Replace with your MySQL root password
    let pool = Pool::new(mysql_url).unwrap();

    loop {
        // Clear the blocks table before fetching new data
        clear_table(&pool).unwrap(); // Call the function to clear the table

        // Fetch the block count from Bitcoin Core
        let client = Client::new();
        let resp = client
            .post(url)
            .basic_auth(rpc_user, Some(rpc_password))
            .json(&json!({
                "jsonrpc": "1.0",
                "id": "curltest",
                "method": "getblockcount",
                "params": []
            }))
            .send()
            .await
            .unwrap();

        let json_response: serde_json::Value = resp.json().await.unwrap();
        let latest_block_count = json_response["result"].as_i64().unwrap();

        // Fetch the last 10 blocks using the latest block count
        let mut block_info_vec = Vec::new();

        // Inside the loop that fetches blocks
        for i in 0..10 {
            let block_index = latest_block_count - i;
            
            // First, get the block hash using the block index
            let block_hash_resp = client
                .post(url)
                .basic_auth(rpc_user, Some(rpc_password))
                .json(&json!({
                    "jsonrpc": "1.0",
                    "id": "curltest",
                    "method": "getblockhash",
                    "params": [block_index]
                }))
                .send()
                .await
                .unwrap();
        
            let block_hash_info: serde_json::Value = block_hash_resp.json().await.unwrap();
        
            // Log the response for debugging
            println!("Response for block hash {}: {:?}", block_index, block_hash_info);
        
            // Check if block hash data is present
            if let Some(result) = block_hash_info.get("result") {
                if let Some(block_hash) = result.as_str() {
                    // Now, use the block hash to get block details
                    let block_resp = client
                        .post(url)
                        .basic_auth(rpc_user, Some(rpc_password))
                        .json(&json!({
                            "jsonrpc": "1.0",
                            "id": "curltest",
                            "method": "getblock",
                            "params": [block_hash]
                        }))
                        .send()
                        .await
                        .unwrap();
        
                    let block_info: serde_json::Value = block_resp.json().await.unwrap();
        
                    // Check if block data is present
                    if let Some(result) = block_info.get("result") {
                        if let (Some(block_height), Some(timestamp)) = (
                            result.get("height").and_then(|v| v.as_i64()),
                            result.get("time").and_then(|v| v.as_i64()),
                        ) {
                            println!("Block Height: {}, Timestamp: {}", block_height, timestamp);
                            block_info_vec.push((block_height, timestamp));
                        } else {
                            println!("Failed to fetch block height or timestamp for block index: {}", block_index);
                        }
                    } else {
                        println!("Failed to fetch block information for block index: {}", block_index);
                    }
                } else {
                    println!("Failed to get block hash for index: {}", block_index);
                }
            } else {
                println!("Failed to fetch block hash information for block index: {}", block_index);
            }
        }
        


        // Insert data into MySQL if we have valid data
        let mut conn = pool.get_conn().unwrap();
        for (block_height, timestamp) in block_info_vec {
            // Convert Unix timestamp to NaiveDateTime
            let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);

            conn.exec_drop(
                r"INSERT INTO blocks (block_height, timestamp) VALUES (:block_height, :timestamp)",
                params! {
                    "block_height" => block_height,
                    "timestamp" => naive_datetime, // Insert as NaiveDateTime
                },
            ).unwrap();
        }

        println!("Data for the latest blocks inserted into the 'blocks' table.");

        // Sleep for a defined interval (e.g., 10 seconds)
        let ten_seconds = time::Duration::from_secs(10);
        thread::sleep(ten_seconds);
    }
}
