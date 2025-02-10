import { Program } from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { AddedAccount, startAnchor } from "solana-bankrun";
import { AutomatedMarketMaker } from "../target/types/automated_market_maker";
import idl from "../target/idl/automated_market_maker.json";
import { MINT_SIZE, MintLayout, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { mintX, mintY } from "./constants";

export async function getBankrunSetup(accounts: AddedAccount[] = []) {
  const [mintXData, mintYData] = Array.from({ length: 2 }, () =>
    Buffer.alloc(MINT_SIZE)
  );

  [mintXData, mintYData].forEach((data) => {
    MintLayout.encode(
      {
        decimals: 6,
        freezeAuthority: PublicKey.default,
        freezeAuthorityOption: 0,
        isInitialized: true,
        mintAuthority: PublicKey.default,
        mintAuthorityOption: 0,
        supply: 100n,
      },
      data
    );
  });

  const context = await startAnchor(
    "",
    [],
    [
      ...accounts,
      {
        address: mintX.publicKey,
        info: {
          data: mintXData,
          executable: false,
          lamports: LAMPORTS_PER_SOL,
          owner: TOKEN_PROGRAM_ID,
        },
      },
      {
        address: mintY.publicKey,
        info: {
          data: mintYData,
          executable: false,
          lamports: LAMPORTS_PER_SOL,
          owner: TOKEN_PROGRAM_ID,
        },
      },
    ]
  );

  const provider = new BankrunProvider(context);
  const program = new Program(idl as AutomatedMarketMaker, provider);

  return {
    context,
    provider,
    program,
  };
}
