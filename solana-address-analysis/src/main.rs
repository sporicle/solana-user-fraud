use chrono::{DateTime, Timelike, Utc};
use indicatif::ProgressBar;
use serde::Serialize;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiMessage;
use solana_transaction_status::UiTransactionEncoding;
use std::collections::HashMap;
use std::str::FromStr;

fn main() {
    let url = "https://api.mainnet-beta.solana.com";
    let commitment_config = CommitmentConfig::processed();
    let client = RpcClient::new_with_commitment(url, commitment_config);

    analyze_user_timezone("EQTKNpQkgK7JhaoKxteaM5Jhn2QzWcurYXvk4QUsDtCi", &client);

    // // Contract ID for Lifinity Protocol
    // let contract_id = Pubkey::from_str("AvtfUvU3byPXgp6Dpw3mgKB2BbVwQvGyry9KeMzD9BLc").unwrap();

    // println!(
    //     "[LOG] Getting recent signatures for contract: {}",
    //     contract_id
    // );
    // let res = client.get_signatures_for_address(&contract_id).unwrap();
    // println!("[LOG] Transactions fetched: {:?}", res.len());
    // let bar = ProgressBar::new(res.len().try_into().unwrap());

    // // key: signer | value: [signing timestamps]
    // let mut signer_count = HashMap::new();
    // // for i in 0..res.len() {
    // for i in 0..2 {
    //     bar.inc(1);
    //     let (signers, ts) = get_txn_signers(&client, &res[i].signature);
    //     for signer in signers {
    //         let (count, timestamps) = signer_count.entry(signer).or_insert((0, Vec::new()));
    //         *count += 1;
    //         timestamps.push(ts);
    //     }
    // }
    // bar.finish();
    // let mut count_vec: Vec<_> = signer_count.iter().collect();

    // // [(signer: str, )]
    // count_vec.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    // println!("{:?}", count_vec);
    // println!(
    //     "[LOG] Number of unique signers in the last {} transactions: {}",
    //     res.len(),
    //     count_vec.len()
    // );
    // println!("{:?}", &count_vec[0]);
    // // println!("{:?}", analyze_timestamps(&count_vec[0].1.1));
    // // println!("{:?}", analyze_timestamps(&count_vec[1].1.1));
    // // println!("{:?}", analyze_timestamps(&count_vec[2].1.1));
}

fn get_txn_signers(client: &RpcClient, txn_id: &String) -> (Vec<String>, i32) {
    let mut result = Vec::new();
    let txn = client
        .get_transaction(
            &Signature::from_str(&txn_id).unwrap(),
            UiTransactionEncoding::JsonParsed,
        )
        .unwrap();
    match txn.transaction.transaction {
        EncodedTransaction::Json(value) => match value.message {
            UiMessage::Parsed(parsed_msg) => {
                for account in parsed_msg.account_keys {
                    if account.signer {
                        result.push(account.pubkey);
                    }
                }
            }
            UiMessage::Raw(_raw_msg) => {
                println!("Currently only support parsed messages")
            }
        },
        _ => {
            println!("Unable to decode not Json");
        }
    };

    match txn.block_time {
        Some(timestamp) => (result, timestamp.try_into().unwrap()),
        None => (result, -1),
    }
}

fn analyze_timestamps(times: &Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for i in 1..times.len() {
        result.push(times[i] - times[i - 1]);
    }

    result
}

fn analyze_user_timezone(pubkey: &str, client: &RpcClient) -> i32 {
    let user_pubkey = Pubkey::from_str(pubkey).unwrap();
    let res = client.get_signatures_for_address(&user_pubkey).unwrap();
    let mut usage_counts: HashMap<u32, usize> = HashMap::new();
    // fill usage_counts with 0
    for i in 0..24 {
        usage_counts.insert(i, 0);
    }

    // time analysis.
    for i in 1..res.len() {
        let unix_time = res[i].block_time.unwrap();
        let utc_datetime: DateTime<Utc> = DateTime::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(unix_time, 0).unwrap(),
            Utc,
        );
        let hour = utc_datetime.hour();
        let count = usage_counts.entry(hour).or_insert(0);
        *count += 1;
    }
    let mut count_vec: Vec<_> = usage_counts.iter().collect();
    count_vec.sort_by_key(|&(number, _count)| number);

    // format into json.
    let mut formated_vec: Vec<HashMap<&str, Value>> = Vec::new();
    for (number, count) in count_vec {
        let mut entry = HashMap::new();
        entry.insert("hour", json!(number));
        entry.insert("actions", json!(count));
        formated_vec.push(entry);
    }
    let json_output = serde_json::to_string_pretty(&formated_vec).unwrap();
    println!("{}", json_output);

    // Find the 3 highest action hours.
    let mut highest_hours = Vec::new();
    for i in 0..24 {
        if highest_hours.len() < 3 {
            highest_hours.push(formated_vec[i]["actions"].as_i64().unwrap());
        } else {
            let mut min_index = 0;
            for j in 0..3 {
                if highest_hours[j] < highest_hours[min_index] {
                    min_index = j;
                }
            }
            if formated_vec[i]["actions"].as_i64().unwrap() > highest_hours[min_index] {
                highest_hours[min_index] = formated_vec[i]["actions"].as_i64().unwrap();
            }
        }
    }
    println!("Highest hours: {:?}", highest_hours);

    let low_activity_threshold = ((highest_hours[0] + highest_hours[1] + highest_hours[2]) / 3) / 3;
    println!("Low activity threshold: {}", low_activity_threshold);

    // Calculate if there are 6 continuous hours of low activity.
    let mut sleep_hours = 0;
    let mut sleep_start = 0;
    let mut is_real_user = false;

    for i in 0..24 {
        if formated_vec[i]["actions"].as_i64().unwrap() < low_activity_threshold {
            if sleep_hours == 0 {
                sleep_start = i;
            }
            sleep_hours += 1;
            if sleep_hours == 6 {
                println!("Sleep hours: {} - {}", sleep_start, i);
                is_real_user = true;
            }
        } else {
            sleep_hours = 0;
        }
    }

    println!("Sleep Start: {}", sleep_start);
    println!("Is real user: {}", is_real_user);
    return is_real_user as i32;
}
