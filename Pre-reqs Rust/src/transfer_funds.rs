use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
    system_instruction::transfer,
    pubkey::Pubkey,
};
use std::str::FromStr;

const RPC_URL: &str = "https://api.devnet.solana.com";

fn main() {
    let rpc_client = RpcClient::new(RPC_URL);
    let keypair = read_keypair_file("./Turbin3-wallet.json").expect("Couldn't find wallet file");

    let from_account = Pubkey::from_str("B6MoAAVinv3nwd9wMLSNJtHAFDxzgNdMVANhusmUE41t").unwrap();
    let to_account = Pubkey::from_str("GsEukyDE1v9WfpDfPAtmZwYbiSRzqihs4TxaLQK7vCNT").unwrap();

    let balance = rpc_client.get_balance(&from_account).expect("Failed to get balance");

    if balance > 0 {
        println!("Transferring {} lamports from {} to {}", balance, from_account, to_account);

        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&from_account, &to_account, balance)],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    } else {
        println!("No funds available to transfer.");
    }
}
