import { BankrunProvider } from "anchor-bankrun";
import { beforeEach, describe, expect, test } from "bun:test";
import { ProgramTestContext } from "solana-bankrun";
import { AutomatedMarketMaker } from "../../target/types/automated_market_maker";
import { BN, Program } from "@coral-xyz/anchor";
import { getBankrunSetup } from "../setup";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { mintX, mintY } from "../constants";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { getConfigPdaAndBump, getMintLpPdaAndBump } from "../pda";
import { getConfigAcc } from "../accounts";

describe("initialize", () => {
  let { context, provider, program } = {} as {
    context: ProgramTestContext;
    provider: BankrunProvider;
    program: Program<AutomatedMarketMaker>;
  };

  const authority = Keypair.generate();

  beforeEach(async () => {
    ({ context, provider, program } = await getBankrunSetup([
      {
        address: authority.publicKey,
        info: {
          lamports: LAMPORTS_PER_SOL * 5,
          data: Buffer.alloc(0),
          owner: SystemProgram.programId,
          executable: false,
        },
      },
    ]));
  });

  test("initialize a pool config", async () => {
    const seed = new BN(randomBytes(8));
    const locked = false;
    const fee = 100;

    await program.methods
      .initialize({
        seed,
        locked,
        fee,
      })
      .accounts({
        authority: authority.publicKey,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();

    const [configPda, configBump] = getConfigPdaAndBump(seed);

    const configAcc = await getConfigAcc(program, configPda);
    const [mintLpPda, mintLpBump] = getMintLpPdaAndBump(configPda);

    expect(configAcc.seed).toStrictEqual(seed);
    expect(configAcc.locked).toEqual(locked);
    expect(configAcc.bump).toEqual(configBump);
    expect(configAcc.lpBump).toEqual(mintLpBump);
    expect(configAcc.fee).toEqual(fee);
    expect(configAcc.mintX).toStrictEqual(mintX.publicKey);
    expect(configAcc.mintY).toStrictEqual(mintY.publicKey);
    expect(configAcc.authority).toStrictEqual(authority.publicKey);

    const mintLpAcc = await context.banksClient.getAccount(mintLpPda);

    expect(mintLpAcc).not.toBeNull();

    const vaultX = getAssociatedTokenAddressSync(
      mintX.publicKey,
      configPda,
      true,
      TOKEN_PROGRAM_ID,
    );
    const vaultY = getAssociatedTokenAddressSync(
      mintY.publicKey,
      configPda,
      true,
      TOKEN_PROGRAM_ID,
    );

    const vaultXAcc = await context.banksClient.getAccount(vaultX);
    const vaultYAcc = await context.banksClient.getAccount(vaultY);

    expect(vaultXAcc).not.toBeNull();
    expect(vaultYAcc).not.toBeNull();
  });
});
