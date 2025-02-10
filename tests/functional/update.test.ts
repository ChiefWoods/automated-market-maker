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
      .initialize({
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
  });

  test("update a pool config", async () => {
    const locked = true;
    const fee = 200;
    const authority = authorityB.publicKey;

    await program.methods
      .update({
        locked,
        fee,
        authority,
      })
      .accounts({
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
        .update({
          locked,
          fee,
          authority,
        })
        .accounts({
          config: configPda,
        })
        .signers([authorityB])
        .rpc();
    } catch (err) {
      expect(err).toBeInstanceOf(AnchorError);
      expect(err.errorCode.error.code).toEqual("InvalidConfigAuthority");
      expect(err.errorCode.error.number).toEqual(6000);
    }
  });
});
