use indicatif::ProgressBar;
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

    // Contract ID for Lifinity Protocol
    let contract_id = Pubkey::from_str("AvtfUvU3byPXgp6Dpw3mgKB2BbVwQvGyry9KeMzD9BLc").unwrap();

    let commitment_config = CommitmentConfig::processed();
    let client = RpcClient::new_with_commitment(url, commitment_config);

    println!(
        "[LOG] Getting recent signatures for contract: {}",
        contract_id
    );
    let res = client.get_signatures_for_address(&contract_id).unwrap();
    println!("[LOG] Transactions fetched: {:?}", res.len());
    let bar = ProgressBar::new(res.len().try_into().unwrap());

    // key: signer | value: [signing timestamps] 
    let mut signer_count = HashMap::new();
    // for i in 0..res.len() {
    for i in 0..2 {
        bar.inc(1);
        let (signers, ts) = get_txn_signers(&client, &res[i].signature);
        for signer in signers {
            let (count, timestamps) = signer_count.entry(signer).or_insert((0, Vec::new()));
            *count += 1;
            timestamps.push(ts);
        }
    }
    bar.finish();
    let mut count_vec: Vec<_> = signer_count.iter().collect();

    // [(signer: str, )]
    count_vec.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    println!("{:?}", count_vec);
    println!(
        "[LOG] Number of unique signers in the last {} transactions: {}",
        res.len(),
        count_vec.len()
    );
    println!("{:?}", &count_vec[0]);
    // println!("{:?}", analyze_timestamps(&count_vec[0].1.1));
    // println!("{:?}", analyze_timestamps(&count_vec[1].1.1));
    // println!("{:?}", analyze_timestamps(&count_vec[2].1.1));
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
            },
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
