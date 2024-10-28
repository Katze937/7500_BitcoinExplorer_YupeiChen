use std::{thread, time};
use mysql::*;
use mysql::prelude::*;
use reqwest::Client;
use serde_json::json;
use chrono::{NaiveDateTime, Utc, TimeZone};

struct BlockInfo {
    block_height: i64,
    timestamp: i64, // Store as i64 to convert to NaiveDateTime later
    tx_count: i64,
    block_size: i64,
    fees: f64,
    miner: String,
    avg_tx_size: i64,
}

fn clear_table(pool: &Pool) -> mysql::Result<()> {
    let mut conn = pool.get_conn()?;
    conn.query_drop("TRUNCATE TABLE blocks")?;
    println!("Table 'blocks' cleared and AUTO_INCREMENT reset.");
    Ok(())
}

#[tokio::main]
async fn main() {
    let rpc_user = "myusername"; // Replace with your Bitcoin Core RPC username
    let rpc_password = "mypassword"; // Replace with your Bitcoin Core RPC password
    let url = "http://127.0.0.1:8332/";

    let mysql_url = "mysql://root:Aisa937!@127.0.0.1:3306/bitcoin_data"; // Replace with your MySQL password
    let pool = Pool::new(mysql_url).unwrap();

    loop {
        let client = Client::new();
        
        // Step 1: Fetch latest block count
        let resp = client
            .post(url)
            .basic_auth(rpc_user, Some(rpc_password))
            .json(&json!( {
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

        let mut block_info_vec: Vec<BlockInfo> = Vec::new();

        // Step 2: Fetch data for the latest blocks
        for i in 0..10 {
            let block_index = latest_block_count - i;

            let block_hash_resp = client
                .post(url)
                .basic_auth(rpc_user, Some(rpc_password))
                .json(&json!( {
                    "jsonrpc": "1.0",
                    "id": "curltest",
                    "method": "getblockhash",
                    "params": [block_index]
                }))
                .send()
                .await
                .unwrap();

            let block_hash_info: serde_json::Value = block_hash_resp.json().await.unwrap();
            println!("Block Hash Info Response: {:?}", block_hash_info);

            if let Some(result) = block_hash_info.get("result") {
                if let Some(block_hash) = result.as_str() {
                    let block_resp = client
                        .post(url)
                        .basic_auth(rpc_user, Some(rpc_password))
                        .json(&json!( {
                            "jsonrpc": "1.0",
                            "id": "curltest",
                            "method": "getblock",
                            "params": [block_hash, 2]
                        }))
                        .send()
                        .await
                        .unwrap();

                    let block_info: serde_json::Value = block_resp.json().await.unwrap();

                    if let Some(result) = block_info.get("result") {
                        let block_height = result.get("height").and_then(|v| v.as_i64()).unwrap_or(0);
                        let timestamp = result.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
                        let tx_count = result.get("nTx").and_then(|v| v.as_i64()).unwrap_or(0);
                        let block_size = result.get("size").and_then(|v| v.as_i64()).unwrap_or(0);
                        let fees = result.get("fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let miner = result.get("miner").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let avg_tx_size = result.get("avgTxSize").and_then(|v| v.as_i64()).unwrap_or(0);

                        block_info_vec.push(BlockInfo {
                            block_height,
                            timestamp,
                            tx_count,
                            block_size,
                            fees,
                            miner,
                            avg_tx_size,
                        });
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

        // Step 3: Clear the table after fetching data
        clear_table(&pool).unwrap();

        // Step 4: Insert fetched data into the database
        let mut conn = pool.get_conn().unwrap();
        for block in block_info_vec {
            let naive_datetime = NaiveDateTime::from_timestamp_opt(block.timestamp, 0)
                .expect("Failed to convert timestamp to NaiveDateTime");

            conn.exec_drop(
                r"INSERT INTO blocks (block_height, timestamp, transaction_count, block_size, fees_collected, miner_address, avg_transaction_size) 
                  VALUES (:block_height, :timestamp, :transaction_count, :block_size, :fees_collected, :miner_address, :avg_transaction_size)",
                params! {
                    "block_height" => block.block_height,
                    "timestamp" => naive_datetime,
                    "transaction_count" => block.tx_count,
                    "block_size" => block.block_size,
                    "fees_collected" => block.fees,
                    "miner_address" => block.miner,
                    "avg_transaction_size" => block.avg_tx_size,
                },
            ).unwrap();
        }

        println!("Data for the latest blocks inserted into the 'blocks' table.");

        let ten_seconds = time::Duration::from_secs(10);
        thread::sleep(ten_seconds);
    }
}
