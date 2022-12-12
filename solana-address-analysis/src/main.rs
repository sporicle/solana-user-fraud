use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiMessage;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use std::collections::HashMap;
use indicatif::ProgressBar;

fn main() {
    let url = "https://api.mainnet-beta.solana.com";

    // Contract ID for Lifinity Protocol
    let contract_id = Pubkey::from_str("AvtfUvU3byPXgp6Dpw3mgKB2BbVwQvGyry9KeMzD9BLc").unwrap();

    let commitment_config = CommitmentConfig::processed();
    let client = RpcClient::new_with_commitment(url, commitment_config);

    println!("[LOG] Getting recent signatures for contract: {}", contract_id);
    let res = client.get_signatures_for_address(&contract_id).unwrap();
    println!("[LOG] Transactions fetched: {:?}", res.len());
    let bar = ProgressBar::new(res.len().try_into().unwrap());

    let mut signer_count = HashMap::new();
    for i in 0..res.len() {
        bar.inc(1);
        let (signers, timestamp) = get_txn_signers(&client, &res[i].signature);
        for signer in signers {
            let (count, _timestamp) = signer_count.entry(signer).or_insert((0, timestamp));
            *count +=1;
        }
    }
    bar.finish();
    let mut count_vec: Vec<_> = signer_count.iter().collect();
    count_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    println!("{:?}", count_vec);
    println!("[LOG] Number of unique signers in the last {} transactions: {}", res.len(), count_vec.len());
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

    (result, 1) 
}
