import { web3, AnchorProvider, Wallet, BN } from "@coral-xyz/anchor";
import { MushiStakeVaultProgramRpc } from "./mushiStakeVaultProgramRpc";
import { createEclipseTokenMint, createMushiTokenMint } from "./createTokens";

async function main() {
  // Setup connection and wallet
  const connection = new web3.Connection("http://localhost:8899", "confirmed");
  // Load your wallet keypair from a file or create a new one for testing
  const keypairBuffer = require('fs').readFileSync('/path/to/your/keypair.json');
  const walletKeypair = web3.Keypair.fromSecretKey(
    Buffer.from(JSON.parse(keypairBuffer.toString()))
  );
  // For testing, you can also just generate a random keypair:
  // const walletKeypair = web3.Keypair.generate();
  
  // Fund the wallet if needed (for local testing)
  const balance = await connection.getBalance(walletKeypair.publicKey);
  if (balance < web3.LAMPORTS_PER_SOL) {
    console.log("Airdropping 2 SOL...");
    const signature = await connection.requestAirdrop(
      walletKeypair.publicKey,
      2 * web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(signature);
  }

  const wallet = new Wallet(walletKeypair);
  
  // Your program ID
  const programId = new web3.PublicKey("YOUR_PROGRAM_ID");
  
  // Initialize the MushiStakeVaultProgramRpc client
  const client = new MushiStakeVaultProgramRpc({
    rpc: "http://localhost:8899",
    wallet,
    programId,
  });

  // Step 1: Create the Mushi token mint (standard Token Program)
  console.log("Creating Mushi Token Mint...");
  const mushiTokenMint = await createMushiTokenMint(
    connection,
    walletKeypair,
    walletKeypair.publicKey
  );

  // Step 2: Create the Eclipse token mint (Token-2022 Program)
  console.log("Creating Eclipse Token Mint...");
  const eclipseTokenMint = await createEclipseTokenMint(
    connection,
    walletKeypair,
    walletKeypair.publicKey
  );

  // Step 3: Initialize the stake vault
  console.log("Initializing Stake Vault...");
  const initResult = await client.initialize({
    stakeTokenName: "Staked Mushi Token",
    stakeTokenSymbol: "stMUSHI",
    stakeTokenUri: "https://example.com/stmushi-metadata.json",
    mushiTokenMint,
    eclipseTokenMint,
  });

  if (initResult.isPass) {
    console.log("Initialization successful!");
    console.log("Transaction signature:", initResult.info.txSignature);
    
    // Fetch and display the main state for verification
    const mainState = await client.getMainState();
    console.log("Main State:", mainState);
  } else {
    console.error("Initialization failed:", initResult.info);
  }
}

main().catch(console.error); 