import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { MushiStakeVault } from "../target/types/mushi_stake_vault";
import { MushiStakeVaultProgramRpc, sleep } from "./mushiStakeVaultProgramRpc";
import * as dotenv from 'dotenv';
import * as path from 'path';

// Load environment variables from .env file
dotenv.config({ path: path.resolve(__dirname, '../.env') });

describe("mushi_stake_vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const rpc = connection.rpcEndpoint;

  const programId = process.env.PROGRAM_ID 
    ? new web3.PublicKey(process.env.PROGRAM_ID) 
    : new web3.PublicKey("Bne2XHWW1HaMVHp6jXmCcmX3dVrtFMoYV5n2eyrvFw46");
  
  const mushiProgramId = process.env.MUSHI_PROGRAM_ID 
    ? new web3.PublicKey(process.env.MUSHI_PROGRAM_ID) 
    : new web3.PublicKey("HF5x1bCgynzEnBL7ATMFYPNFjBaqfxgMASyUJL2ud6Xi");
  let connectivity = new MushiStakeVaultProgramRpc({
    rpc,
    wallet: provider.wallet,
    programId
  })

  const mushiTokenMint = new web3.PublicKey(process.env.MUSHI_TOKEN_MINT || "");
  const eclipseTokenMint = new web3.PublicKey(process.env.ECLIPSE_TOKEN_MINT || "");

  it("Is initialized!", async () => {
    const initRes = await connectivity.initialize({
      stakeTokenName: "Mushi Stake Token",
      stakeTokenSymbol: "IMUSHI",
      stakeTokenUri: "https://mushi.xyz/stake",
      mushiTokenMint,
      eclipseTokenMint,
      mushiProgramId,
    });
    if (!initRes.isPass) throw "failed to init mainstate";
    await sleep(15_000);
    const info = await connectivity.getMainState();
    if (!info) throw "failed to get mainstate info";
    console.log(info);
  });
});
