import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { GetVersionedTransactionConfig, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";
import { assert } from "chai";

const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor.getProvider().connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    'confirmed'
  )
  return signature
}


/* const transactionDetails = await anchor.getProvider().connection.getTransaction(tx, txConfig);

        // Check if transaction details were returned successfully
        if (transactionDetails) {
            // Calculate the fee spent on this transaction
            const fee = transactionDetails.meta.fee;
            console.log("Transaction Fee:", fee);
        } else {
            console.log("Transaction details not found.");
        }
}; */

describe("vault test", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Vault as Program<Vault>;

  const user = Keypair.generate();

  let rentFeeOnStatePDA = 0;

  const [state, stateBump] = PublicKey.findProgramAddressSync([Buffer.from("state"), user.publicKey.toBytes()], program.programId);
  console.log("state: ", state.toBase58());
  console.log("stateBump: ", stateBump);
  const [vault, vaultBump] = PublicKey.findProgramAddressSync([Buffer.from("vault"), state.toBytes()], program.programId);
  console.log("vault: ", vault.toBase58());
  console.log("vaultBump: ", vaultBump);

  it("set user with sol", async () => {

    const airdropTx = await anchor.getProvider().connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL).then(confirmTx);

    console.log("airdrop tx signature", airdropTx);

   const balance = await anchor.getProvider().connection.getBalance(user.publicKey);
    console.log("balance: ", balance);
  })

  it("Initialize vault", async () => {

    const tx = await program.methods.initialize().accountsPartial({
      user: user.publicKey,
      state,
      vault,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).transaction();

    const simulate = await anchor.getProvider().simulate(tx);
    console.log("simulate: ", simulate);
    console.log("simulate: ", simulate.unitsConsumed);
    const fee = simulate.unitsConsumed;
    
    const initTx = await program.methods.initialize().accountsPartial({
      user: user.publicKey,
      state,
      vault,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([user]).rpc().then(confirmTx);

    const userBalance = await anchor.getProvider().connection.getBalance(user.publicKey);
    console.log("user balance: ", userBalance);

    const stateBalance = await anchor.getProvider().connection.getBalance(state);
    rentFeeOnStatePDA = stateBalance;
    console.log("state balance: ", stateBalance);
  });

  it("Deposit vault", async () => {
    
    const depositTx = await program.methods.deposit(new BN(5 * LAMPORTS_PER_SOL)).accountsPartial({
      user: user.publicKey,
      state,
      vault,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([user]).rpc().then(confirmTx);

    console.log("Deposit transaction signature", depositTx);

    const balance = await anchor.getProvider().connection.getBalance(vault);
    console.log("vault balance: ", balance);
    assert.equal(balance, 5 * LAMPORTS_PER_SOL);
    
  });

  it("Withdraw partial amount from vault", async () => {
    const withdrawPartialValue = await program.methods.withdraw(new BN(3 * LAMPORTS_PER_SOL)).accountsPartial({
      user: user.publicKey,
      state,
      vault,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([user]).rpc().then(confirmTx);

    console.log("Withdraw partial transaction signature", withdrawPartialValue);

    const balance = await anchor.getProvider().connection.getBalance(vault);
    console.log("vault balance: ", balance);
    assert.equal(balance, 2 * LAMPORTS_PER_SOL);
  });

 
  it("close account", async()=>{

    const closeTx = await program.methods.close().accountsPartial({
        user: user.publicKey,
        state,
        vault,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    ).signers([user]).rpc().then(confirmTx);

    console.log("close transaction signature", closeTx);

    const balance = await anchor.getProvider().connection.getBalance(vault);
    console.log("vault balance: ", balance);
    assert.equal(balance, 0);

    let userBalance = await anchor.getProvider().connection.getBalance(user.publicKey);
    console.log("user balance: ", userBalance);
    assert.equal(userBalance + rentFeeOnStatePDA, 10 * LAMPORTS_PER_SOL);


    const closeStateTx = await program.methods.closeState().accountsPartial({
      user: user.publicKey,
        state,
        systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([user]).rpc().then(confirmTx);

    userBalance = await anchor.getProvider().connection.getBalance(user.publicKey);
    console.log("user balance: ", userBalance);

    console.log("close transaction signature", closeStateTx);
    assert.equal(userBalance, 10 * LAMPORTS_PER_SOL);

  });
});
