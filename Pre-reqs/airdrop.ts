import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import wallet from "./mipswallet.json"; // Ensure wallet.json is in the same directory

// Import the keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

// Establish a connection to the Solana devnet
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// Claim 2 devnet SOL tokens
(async () => {
    try {
        console.log(`Requesting 2 SOL for wallet: ${keypair.publicKey.toBase58()}`);
        const txhash = await connection.requestAirdrop(keypair.publicKey, 2 * LAMPORTS_PER_SOL);

        console.log(`Success! Check out your TX here:
https://explorer.solana.com/tx/${txhash}?cluster=devnet`);
    } catch (e) {
        console.error(`Oops, something went wrong: ${e}`);
    }
})();
