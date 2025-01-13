import { Transaction, SystemProgram, Connection, Keypair, LAMPORTS_PER_SOL, sendAndConfirmTransaction, PublicKey } from "@solana/web3.js";
import wallet from "./wallet.json";  // Import your dev wallet keypair from wallet.json

// Import the dev wallet keypair
const from = Keypair.fromSecretKey(new Uint8Array(wallet));

// Define the Turbin3 public key
const to = new PublicKey("GLtaTaYiTQrgz411iPJD79rsoee59HhEy18rtRdrhEUJ");

// Create a Solana devnet connection
const connection = new Connection("https://api.devnet.solana.com");

(async () => {
    try {
        // Get the balance of the dev wallet (in lamports)
        const balance = await connection.getBalance(from.publicKey);
        console.log(`Dev wallet balance: ${balance} lamports`);

        // Create a test transaction to calculate the fee
        const transaction = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: from.publicKey,
                toPubkey: to,
                lamports: balance,
            })
        );

        // Set the blockhash and fee payer for the transaction
        transaction.recentBlockhash = (await connection.getLatestBlockhash('confirmed')).blockhash;
        transaction.feePayer = from.publicKey;

        // Calculate the transaction fee
        const fee = (await connection.getFeeForMessage(transaction.compileMessage(), 'confirmed')).value || 0;
        console.log(`Transaction fee: ${fee} lamports`);

        // Remove the initial transfer instruction to replace it with the adjusted transfer amount
        transaction.instructions.pop();

        // Add the transfer instruction again, but with the balance minus the fee to cover the transaction
        transaction.add(
            SystemProgram.transfer({
                fromPubkey: from.publicKey,
                toPubkey: to,
                lamports: balance - fee,  // Ensure enough SOL is left to pay for the transaction fee
            })
        );

        // Sign, send, and confirm the transaction
        const signature = await sendAndConfirmTransaction(connection, transaction, [from]);
        console.log(`Success! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);

    } catch (e) {
        console.error(`Oops, something went wrong: ${e}`);
    }
})();
