import { BankrunProvider } from "anchor-bankrun";
import { beforeEach, describe, expect, test } from "bun:test";
import { ProgramTestContext } from "solana-bankrun";
import { AutomatedMarketMaker } from "../../target/types/automated_market_maker";
import { AnchorError, BN, Program } from "@coral-xyz/anchor";
import { getBankrunSetup } from "../setup";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { mintX, mintY } from "../constants";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { getConfigPdaAndBump } from "../pda";
import { getConfigAcc } from "../accounts";

describe("update", () => {
  let { context, provider, program } = {} as {
    context: ProgramTestContext;
    provider: BankrunProvider;
    program: Program<AutomatedMarketMaker>;
  };

  const [authorityA, authorityB] = Array.from({ length: 2 }, Keypair.generate);
  const seed = new BN(randomBytes(8));
  const [configPda] = getConfigPdaAndBump(seed);

  beforeEach(async () => {
    ({ context, provider, program } = await getBankrunSetup(
      [authorityA, authorityB].map((kp) => ({
        address: kp.publicKey,
        info: {
          lamports: LAMPORTS_PER_SOL * 5,
          data: Buffer.alloc(0),
          owner: SystemProgram.programId,
          executable: false,
        },
      }))
    ));

    await program.methods
      .initializeConfig({
        seed,
        locked: false,
        fee: 100,
      })
      .accounts({
        authority: authorityA.publicKey,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authorityA])
      .rpc();

    await program.methods
      .initializeVaults()
      .accountsPartial({
        authority: authorityA.publicKey,
        config: configPda,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authorityA])
      .rpc();
  });

  test("update a pool config", async () => {
    const locked = true;
    const fee = 200;
    const authority = authorityB.publicKey;

    await program.methods
      .updateConfig({
        locked,
        fee,
        authority,
      })
      .accountsPartial({
        authority: authorityA.publicKey,
        config: configPda,
      })
      .signers([authorityA])
      .rpc();

    const configAcc = await getConfigAcc(program, configPda);

    expect(configAcc.locked).toEqual(locked);
    expect(configAcc.fee).toEqual(fee);
    expect(configAcc.authority).toStrictEqual(authority);
  });

  test("throws if signer is not config authority", async () => {
    const locked = true;
    const fee = 200;
    const authority = authorityB.publicKey;

    try {
      await program.methods
        .updateConfig({
          locked,
          fee,
          authority,
        })
        .accountsPartial({
          authority: authorityB.publicKey,
          config: configPda,
        })
        .signers([authorityB])
        .rpc();
    } catch (err) {
      expect(err).toBeInstanceOf(AnchorError);

      const { error } = err as AnchorError;
      expect(error.errorCode.code).toEqual("InvalidConfigAuthority");
      expect(error.errorCode.number).toEqual(6000);
    }
  });
});
