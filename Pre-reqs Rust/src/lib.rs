mod programs;

#[cfg(test)]
mod tests {
    use solana_sdk::signature::{Keypair, Signer, read_keypair_file};
    use solana_client::rpc_client::RpcClient;
    use solana_program::{pubkey::Pubkey, system_instruction::transfer};
    use solana_sdk::{transaction::Transaction, message::Message};
    use crate::programs::turbin3_prereq::{Turbin3PrereqProgram, CompleteArgs};
    use solana_sdk::system_program;
    use bs58;
    use std::io::{self, BufRead};
    use std::str::FromStr;

    const RPC_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string());
        println!();

        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
fn decode_key_and_use() {
    use bs58;

    let base58_key = "GsEukyDE1v9WfpDfPAtmZwYbiSRzqihs4TxaLQK7vCNT"; // Replace with your Base58 private key
    let bytes = bs58::decode(base58_key).into_vec().expect("Failed to decode Base58");
    println!("Decoded private key bytes: {:?}", bytes);

    // If you want to save the bytes to a file, you can add that here
}


    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        if let Some(Ok(base58)) = stdin.lock().lines().next() {
            match bs58::decode(base58).into_vec() {
                Ok(wallet) => println!("Your wallet file is:\n{:?}", wallet),
                Err(e) => eprintln!("Failed to decode base58: {}", e),
            }
        } else {
            eprintln!("Failed to read input");
        }
    }

    #[test]
    fn wallet_to_base58() {
        use std::io::Cursor;
    
        let mock_input = "[34, 46, 55, 124, 141, 190, 24, 204]"; // Replace with valid wallet bytes
        let stdin = Cursor::new(mock_input);
    
        let wallet = stdin
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
    
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }
    

    #[test]
    fn airdrop_sol() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");

        let client = RpcClient::new(RPC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!(
                    "https://explorer.solana.com/tx/{}?cluster=devnet",
                    s.to_string()
                );
            }
            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        }
    }

    #[test]
    fn transfer_sol() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");

        let to_pubkey = Pubkey::from_str("GsEukyDE1v9WfpDfPAtmZwYbiSRzqihs4TxaLQK7vCNT").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 1_000_000)], // Transfer 0.1 SOL (1 million lamports)
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn empty_wallet() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
    
        let to_pubkey = Pubkey::from_str("GsEukyDE1v9WfpDfPAtmZwYbiSRzqihs4TxaLQK7vCNT").unwrap();
    
        let rpc_client = RpcClient::new(RPC_URL);
    
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
    
        println!("Wallet Public Key: {}", keypair.pubkey());
        println!("Wallet Balance: {}", balance);
    
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
    
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );
    
        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");
    
        println!("Transaction Fee: {}", fee);
        println!("Lamports to Send: {}", balance - fee);
    
        if balance <= fee {
            panic!("Insufficient funds to cover transaction fee");
        }
    
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );
    
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
    
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn submit_turbin3_completion() {
        let rpc_client = RpcClient::new(RPC_URL);
        let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");
    
        let prereq = Turbin3PrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);
    
        if let Ok(account) = rpc_client.get_account(&prereq) {
            println!("PDA Account already exists: {}", prereq);
            println!("Account data: {:?}", account.data);
    
            // Decode GitHub username
            let github_data = &account.data[12..32]; // Adjust indices based on data layout
            if github_data == b"mips-dev" {
                println!("GitHub username already submitted.");
                return;
            }
        }
    
        // If not already submitted, proceed with the transaction
        let args = CompleteArgs {
            github: b"mips-dev".to_vec(),
        };
    
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
    
        let transaction = Turbin3PrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );
    
        match rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                println!(
                    "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
                    signature
                );
            }
            Err(err) => {
                eprintln!("Failed to send transaction: {:?}", err);
            }
        }
    }
    
    

}
