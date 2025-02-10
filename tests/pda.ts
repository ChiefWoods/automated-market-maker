import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import idl from "../target/idl/automated_market_maker.json";

export function getConfigPdaAndBump(seed: BN) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    new PublicKey(idl.address)
  );
}

export function getMintLpPdaAndBump(configPda: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), configPda.toBuffer()],
    new PublicKey(idl.address)
  );
}
