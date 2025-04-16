import { BN, Program, web3 } from "@coral-xyz/anchor";
import { IDL, MushiStakeVault } from "../target/types/mushi_stake_vault";
import { AnchorProvider, Wallet } from "@coral-xyz/anchor/dist/cjs/provider";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { delay } from "./utils";
import { safeAirdrop } from "./utils";

const Seeds = {
  mainState: Buffer.from("main_state"),
  vaultOwner: Buffer.from("vault_owner"),
};

const log = console.log;
export type Result<T, E = string> =
  | { isPass: true; info: T }
  | { isPass: false; info: E };

export type SendTxResult = Result<{ txSignature: string }, string>;
export const TOKEN_DECIMALS_HELPER = 1_000_000_000; // 9 decimals
export const SOL_DECIMALS_HELPER = 1_000_000_000; // 9 decimals
const SECONDS_IN_A_DAY = 86400;
const associatedTokenProgram = ASSOCIATED_TOKEN_PROGRAM_ID;
const mplProgram = new web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const systemProgram = web3.SystemProgram.programId;
const sysvarRent = web3.SYSVAR_RENT_PUBKEY;
const tokenProgram = TOKEN_PROGRAM_ID;
const token2022Program = TOKEN_2022_PROGRAM_ID;

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export type MainStateInfo = {
  admin: web3.PublicKey;
  mushiTokenAmount: number;
  eclipseTokenAmount: number;
  stakingTokenTotalSupply: number;
  mushiTokenMint: web3.PublicKey;
  eclipseTokenMint: web3.PublicKey;
  stakeTokenMint: web3.PublicKey;
};

export class MushiStakeVaultProgramRpc {
  private program: Program<MushiStakeVault>;
  private connection: web3.Connection;
  private programId: web3.PublicKey;
  private mainState: web3.PublicKey;
  private globalState: web3.PublicKey;
  private vaultOwner: web3.PublicKey;
  private provider: AnchorProvider;

  constructor({
    rpc, 
    wallet,
    programId,
  }: {
    rpc: string;
    wallet: Wallet;
    programId: web3.PublicKey;
  }) {
    this.connection = new web3.Connection(rpc);
    const provider = new AnchorProvider(this.connection, wallet, {
      commitment: "confirmed",
    });
    this.provider = provider;
    this.programId = programId;
    this.program = new Program(IDL, programId, provider);
    this.mainState = web3.PublicKey.findProgramAddressSync(
      [Seeds.mainState],
      this.programId
    )[0];
    this.vaultOwner = web3.PublicKey.findProgramAddressSync(
      [Seeds.vaultOwner],
      this.programId
    )[0];
  }

  
  async sendTx(
    ixs: web3.TransactionInstruction[],
    signers?: web3.Keypair[]
  ): Promise<string | null> {
    try {
      const payerKey = this.provider.publicKey;
      const recentBlockhash = (await this.connection.getLatestBlockhash())
        .blockhash;
      const msg = new web3.TransactionMessage({
        instructions: ixs,
        payerKey,
        recentBlockhash,
      }).compileToV0Message();
      const tx = new web3.VersionedTransaction(msg);
      signers && tx.sign(signers);
      const signedTx = await this.provider.wallet
        .signTransaction(tx)
        .catch(() => null);
      if (!signedTx) throw "failed to sign tx";
      const txSignature = await this.connection.sendRawTransaction(
        signedTx.serialize(),
        { skipPreflight: true }
      );
      let expireCount = 0;
      for (let i = 0; i < 50; ++i) {
        await sleep(2_000);
        const res = await this.connection
          .getSignatureStatus(txSignature)
          .catch(() => null);
        if (res) {
          if (res.value?.err) {
            const simRes = await this.connection
              .simulateTransaction(tx, {
                replaceRecentBlockhash: true,
              })
              .catch(() => null)
              .then((res) => res?.value);
            log({ txSignature });
            log({ simRes });
            log({ txSignatureRes: res.value });
            throw "tx failed";
          }
          return txSignature;
        }
        const isValid = await this.connection
          .isBlockhashValid(recentBlockhash)
          .catch(() => null)
          .then((res) => res?.value);
        if (isValid == false) expireCount += 1;
        if (expireCount >= 2) {
          log({ txSignature });
          throw "tx expired";
        }
      }
      log({ txSignature });
      return null;
    } catch (sendTxError) {
      log({ sendTxError });
      return null;
    }
  }

  async getMainState(): Promise<MainStateInfo | null> {
    try {
      const mainState = await this.program.account.mainState.fetch(this.mainState);
      return {
        admin: mainState.admin,
        mushiTokenAmount: mainState.mushiTokenAmount.toNumber(),
        eclipseTokenAmount: mainState.eclipseTokenAmount.toNumber(),
        stakingTokenTotalSupply: mainState.stakingTokenTotalSupply ? mainState.stakingTokenTotalSupply.toNumber() : 0,
        mushiTokenMint: mainState.mushiTokenMint,
        eclipseTokenMint: mainState.eclipseTokenMint,
        stakeTokenMint: mainState.stakeTokenMint,
      };
    } catch (error) {
      log({ error });
      return null;
    }
  }

  async initialize(input: {
    stakeTokenName: string;
    stakeTokenSymbol: string;
    stakeTokenUri: string;
    mushiTokenMint: web3.PublicKey;
    eclipseTokenMint: web3.PublicKey;
    mushiProgramId: web3.PublicKey;
  }): Promise<SendTxResult> {
    try {
      const {stakeTokenName, stakeTokenSymbol, stakeTokenUri, mushiTokenMint, eclipseTokenMint} = input;

      // Check token mint owners first
      const mushiTokenMintInfo = await this.connection.getAccountInfo(mushiTokenMint);
      if (!mushiTokenMintInfo) {
        throw "mushiTokenMint account not found";
      }
      const mushiTokenMintOwner = mushiTokenMintInfo.owner.toBase58();
      console.log("Mushi Token Mint owner:", mushiTokenMintOwner);
      console.log("Expected Token Program:", tokenProgram.toBase58());
      console.log("Mushi Token Mint owner:", mushiTokenMintInfo);
      if (mushiTokenMintOwner !== tokenProgram.toBase58()) {
        throw "mushiTokenMint must be owned by the standard Token Program, not Token-2022 Program";
      }

      const eclipseTokenMintInfo = await this.connection.getAccountInfo(eclipseTokenMint);
      if (!eclipseTokenMintInfo) {
        throw "eclipseTokenMint account not found";
      }
      const eclipseTokenMintOwner = eclipseTokenMintInfo.owner.toBase58();
      console.log("Eclipse Token Mint owner:", eclipseTokenMintOwner);
      console.log("Expected Token2022 Program:", token2022Program.toBase58());
      
      if (eclipseTokenMintOwner !== token2022Program.toBase58()) {
        throw "eclipseTokenMint must be owned by the Token-2022 Program, not standard Token Program";
      }

      const stakeTokenKp = web3.Keypair.generate();
      const stakeTokenMint = stakeTokenKp.publicKey;
      const admin = this.provider.publicKey;

      const mushiTokenVault = getAssociatedTokenAddressSync(mushiTokenMint, this.vaultOwner, true, tokenProgram);
      const eclipseTokenVault = getAssociatedTokenAddressSync(eclipseTokenMint, this.vaultOwner, true, token2022Program);
      const stakeTokenVault = getAssociatedTokenAddressSync(stakeTokenMint, this.vaultOwner, true, tokenProgram);

      const stakeTokenMetadataAccount = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("metadata"), mplProgram.toBuffer(), stakeTokenMint.toBuffer()],
        mplProgram
      )[0];

      const ix = await this.program.methods
        .initialize({
          stakeTokenName: stakeTokenName, 
          stakeTokenSymbol: stakeTokenSymbol,
          stakeTokenUri: stakeTokenUri,
          mushiProgram: input.mushiProgramId,
        }).accounts({
          admin,
          mainState: this.mainState,
          mushiTokenMint: mushiTokenMint,
          eclipseTokenMint: eclipseTokenMint,
          stakeTokenMint: stakeTokenMint,
          mushiTokenVault: mushiTokenVault,
          eclipseTokenVault: eclipseTokenVault,
          stakeTokenVault: stakeTokenVault,
          tokenVaultOwner: this.vaultOwner,
          stakeTokenMetadataAccount,
          mplProgram,
          tokenProgram,
          token2022Program,
          associatedTokenProgram,
          systemProgram,
          rent: sysvarRent,
        }).instruction();

      const ixs = [ix];
      const txSignature = await this.sendTx(ixs, [stakeTokenKp]);
      if (!txSignature) throw "tx failed";
      return { isPass: true, info: { txSignature } };
    } catch (error) {
      log({ error });
      return { isPass: false, info: error };
    }
  }

  async getBaseContext(): Promise<any> {
    const user = this.provider.publicKey;

    const mainStateInfo = await this.getMainState();
    if (!mainStateInfo) throw "mainStateInfo not found";

    console.log("mainStateInfo", mainStateInfo);

    const mushiTokenVault = getAssociatedTokenAddressSync(mainStateInfo.mushiTokenMint, this.vaultOwner, true, tokenProgram);
    const eclipseTokenVault = getAssociatedTokenAddressSync(mainStateInfo.eclipseTokenMint, this.vaultOwner, true, token2022Program);
    // const stakeTokenVault = getAssociatedTokenAddressSync(mainStateInfo.stakeTokenMint, this.vaultOwner, true, tokenProgram);

    const userMushiTokenAta = getAssociatedTokenAddressSync(mainStateInfo.mushiTokenMint, user, true, tokenProgram);
    const userEclipseTokenAta = getAssociatedTokenAddressSync(mainStateInfo.eclipseTokenMint, user, true, token2022Program);
    const userStakeTokenAta = getAssociatedTokenAddressSync(mainStateInfo.stakeTokenMint, user, true, tokenProgram);

    return {
      user,
      mainState: this.mainState,
      mushiTokenMint: mainStateInfo.mushiTokenMint,
      eclipseTokenMint: mainStateInfo.eclipseTokenMint,
      stakeTokenMint: mainStateInfo.stakeTokenMint,
      mushiTokenVault,
      eclipseTokenVault,
      // stakeTokenVault,
      userMushiTokenAta,
      userEclipseTokenAta,
      userStakeTokenAta,
      tokenVaultOwner: this.vaultOwner,
      associatedTokenProgram,
      tokenProgram,
      token2022Program,
      systemProgram,
      instructionSysvar: web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    };
  }

  async stake(input: {
    amount: number;
  }): Promise<SendTxResult> {
    try {
      const { amount } = input;
      const baseContext = await this.getBaseContext();

      const rawAmount = Math.trunc(amount * SOL_DECIMALS_HELPER)

      const ix = await this.program.methods
        .stake({
          amount: new BN(rawAmount),
        }).accounts({
          ...baseContext,
        }).instruction();

      const ixs = [ix];
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "tx failed";
      return { isPass: true, info: { txSignature } };
    } catch (error) {
      log({ error });
      return { isPass: false, info: error };
    }
  }

  async unstake(input: {
    amount: number;
  }): Promise<SendTxResult> {
    try {
      const { amount } = input;
      const baseContext = await this.getBaseContext();

      const rawAmount = Math.trunc(amount * SOL_DECIMALS_HELPER)

      const ix = await this.program.methods
        .unstake({
          amount: new BN(rawAmount),
        }).accounts({
          ...baseContext,
        }).instruction();

      const ixs = [ix];
      const txSignature = await this.sendTx(ixs);
      if (!txSignature) throw "tx failed";
      return { isPass: true, info: { txSignature } };
    } catch (error) {
      log({ error });
      return { isPass: false, info: error };
    }
  }
}
