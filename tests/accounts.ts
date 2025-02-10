import { Connection, PublicKey } from "@solana/web3.js";
import { AutomatedMarketMaker } from "../target/types/automated_market_maker";
import { Program } from "@coral-xyz/anchor";
import { getAccount } from "@solana/spl-token";

export async function getConfigAcc(
  program: Program<AutomatedMarketMaker>,
  configPda: PublicKey
) {
  return await program.account.config.fetchNullable(configPda);
}
