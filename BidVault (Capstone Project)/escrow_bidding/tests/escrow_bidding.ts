import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import { EscrowBidding } from "../target/types/escrow_bidding";
import { assert } from "chai";

describe("escrow_bidding", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.EscrowBidding as Program<EscrowBidding>;
  const owner = provider.wallet; // owner of the escrow

  // PDA and bump for the escrow account.
  let escrowPDA: web3.PublicKey;
  let escrowBump: number;

  // For bid tests, store bidder and bid account keypairs.
  let bidder: web3.Keypair;
  let bidAccount: web3.Keypair;

  // For the "resolve bid" test, store a second bid.
  let winningBidAccount: web3.Keypair;

  before(async () => {
    // Use the constant seed "escrow" and the owner's public key.
    [escrowPDA, escrowBump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow"), owner.publicKey.toBuffer()],
      program.programId
    );
    console.log("Escrow PDA:", escrowPDA.toBase58());
  });  

  it("Initializes escrow with an initial deposit", async () => {
    const initialDeposit = new anchor.BN(2000000); // 2,000,000 lamports
    await program.methods.initEscrow(initialDeposit)
      .accounts({
        escrow: escrowPDA,
        owner: owner.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPDA);
    assert.ok(escrowAccount.depositedAmount.eq(initialDeposit));
    assert.ok(escrowAccount.lockedAmount.eq(new anchor.BN(0)));
    assert.ok(escrowAccount.owner.equals(owner.publicKey));

    // Log the actual lamports balance of the escrow PDA.
    const accountInfo = await provider.connection.getAccountInfo(escrowPDA);
    console.log("Escrow lamports balance:", accountInfo?.lamports);
  });

  it("Deposits additional funds into escrow", async () => {
    const depositAmount = new anchor.BN(500000); // additional 500,000 lamports
    await program.methods.depositFunds(depositAmount)
      .accounts({
        escrow: escrowPDA,
        owner: owner.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPDA);
    const expectedDeposit = new anchor.BN(2500000); // 2,000,000 + 500,000
    assert.ok(escrowAccount.depositedAmount.eq(expectedDeposit));
  });

  it("Places a bid", async () => {
    // Create a new keypair for the bidder.
    bidder = web3.Keypair.generate();
    // Request an airdrop and wait for confirmation.
    const airdropSig = await provider.connection.requestAirdrop(bidder.publicKey, 1000000000);
    await provider.connection.confirmTransaction(airdropSig);

    // (Optional) Log bidder lamports balance.
    const bidderInfo = await provider.connection.getAccountInfo(bidder.publicKey);
    console.log("Bidder lamports:", bidderInfo?.lamports);

    const bidAmount = new anchor.BN(200000); // bid amount: 200,000 lamports
    bidAccount = web3.Keypair.generate();

    await program.methods.placeBid(bidAmount)
      .accounts({
        escrow: escrowPDA,
        bidder: bidder.publicKey,
        bid: bidAccount.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([bidder, bidAccount])
      .rpc();

    const bidData = await program.account.bid.fetch(bidAccount.publicKey);
    assert.ok(bidData.amount.eq(bidAmount));
    assert.ok(bidData.active === true);

    // Check that the escrow locked amount increased.
    const escrowAccount = await program.account.escrow.fetch(escrowPDA);
    assert.ok(escrowAccount.lockedAmount.eq(bidAmount));
  });

  it("Cancels the bid and refunds funds", async () => {
    // Instead of using a CPI transfer (which fails because the escrow PDA holds data),
    // we perform a manual lamport transfer in the program's cancel_bid instruction.
    await program.methods.cancelBid()
      .accounts({
        escrow: escrowPDA,
        bidder: bidder.publicKey,
        bid: bidAccount.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([bidder])
      .rpc();

    const bidData = await program.account.bid.fetch(bidAccount.publicKey);
    assert.ok(bidData.active === false);

    // After refund, the escrow's deposited_amount decreases by the bid amount.
    const escrowAccount = await program.account.escrow.fetch(escrowPDA);
    // Expected deposited amount: previous 2,500,000 - 200,000 refunded = 2,300,000
    const expectedDeposit = new anchor.BN(2300000);
    assert.ok(escrowAccount.depositedAmount.eq(expectedDeposit));
    // Locked amount should be zero.
    assert.ok(escrowAccount.lockedAmount.eq(new anchor.BN(0)));
  });

  it("Places a bid and then resolves it", async () => {
    // Place a new bid that will be resolved.
    const bidAmount = new anchor.BN(300000); // 300,000 lamports bid
    winningBidAccount = web3.Keypair.generate();
    await program.methods.placeBid(bidAmount)
      .accounts({
        escrow: escrowPDA,
        bidder: bidder.publicKey,
        bid: winningBidAccount.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([bidder, winningBidAccount])
      .rpc();

    let bidData = await program.account.bid.fetch(winningBidAccount.publicKey);
    assert.ok(bidData.amount.eq(bidAmount));
    assert.ok(bidData.active === true);

    // Owner resolves the winning bid.
    await program.methods.resolveBid()
      .accounts({
        escrow: escrowPDA,
        winningBid: winningBidAccount.publicKey,
        owner: owner.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    bidData = await program.account.bid.fetch(winningBidAccount.publicKey);
    assert.ok(bidData.active === false);

    // Escrow should have reduced deposited and locked amounts by bidAmount.
    const escrowAccount = await program.account.escrow.fetch(escrowPDA);
    // Previously, after cancellation, deposited was 2,300,000.
    // After resolving 300,000, deposited decreases to 2,000,000.
    const expectedDeposit = new anchor.BN(2000000); // 2,300,000 - 300,000
    assert.ok(escrowAccount.depositedAmount.eq(expectedDeposit));
    assert.ok(escrowAccount.lockedAmount.eq(new anchor.BN(0)));
  });

  it("Withdraws available funds from escrow", async () => {
    // Calculate available funds: deposited_amount - locked_amount.
    let escrowAccount = await program.account.escrow.fetch(escrowPDA);
    const available = escrowAccount.depositedAmount.sub(escrowAccount.lockedAmount);
    // Withdraw half of available funds.
    const withdrawAmount = available.div(new anchor.BN(2));

    await program.methods.withdrawFunds(withdrawAmount)
      .accounts({
        escrow: escrowPDA,
        owner: owner.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    escrowAccount = await program.account.escrow.fetch(escrowPDA);
    const expectedDeposit = available.sub(withdrawAmount);
    assert.ok(escrowAccount.depositedAmount.eq(expectedDeposit));
  });
});
