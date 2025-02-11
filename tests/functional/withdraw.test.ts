import { BankrunProvider } from "anchor-bankrun";
import { beforeEach, describe, expect, test } from "bun:test";
import { ProgramTestContext } from "solana-bankrun";
import { AutomatedMarketMaker } from "../../target/types/automated_market_maker";
import { AnchorError, BN, Program } from "@coral-xyz/anchor";
import { getBankrunSetup } from "../setup";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { randomBytes } from "crypto";
import { mintX, mintY } from "../constants";
import {
  ACCOUNT_SIZE,
  AccountLayout,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { getConfigPdaAndBump, getMintLpPdaAndBump } from "../pda";

describe("withdraw", () => {
  let { context, provider, program } = {} as {
    context: ProgramTestContext;
    provider: BankrunProvider;
    program: Program<AutomatedMarketMaker>;
  };

  const [admin, user] = Array.from({ length: 2 }, Keypair.generate);
  const [userAtaXPda, userAtaYPda] = [mintX, mintY].map((mint) => {
    return getAssociatedTokenAddressSync(
      mint.publicKey,
      user.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
  });

  const seed = new BN(randomBytes(8));
  const [configPda] = getConfigPdaAndBump(seed);

  beforeEach(async () => {
    const [userAtaXPubkeyData, userAtaYPubkeyData] = Array.from(
      { length: 2 },
      () => Buffer.alloc(ACCOUNT_SIZE)
    );

    AccountLayout.encode(
      {
        amount: 10n,
        closeAuthority: PublicKey.default,
        closeAuthorityOption: 0,
        delegate: PublicKey.default,
        delegateOption: 0,
        delegatedAmount: 0n,
        isNative: 0n,
        isNativeOption: 0,
        mint: mintX.publicKey,
        owner: user.publicKey,
        state: 1,
      },
      userAtaXPubkeyData
    );

    AccountLayout.encode(
      {
        amount: 10n,
        closeAuthority: PublicKey.default,
        closeAuthorityOption: 0,
        delegate: PublicKey.default,
        delegateOption: 0,
        delegatedAmount: 0n,
        isNative: 0n,
        isNativeOption: 0,
        mint: mintY.publicKey,
        owner: user.publicKey,
        state: 1,
      },
      userAtaYPubkeyData
    );

    ({ context, provider, program } = await getBankrunSetup([
      ...[admin, user].map((kp) => ({
        address: kp.publicKey,
        info: {
          lamports: LAMPORTS_PER_SOL * 5,
          data: Buffer.alloc(0),
          owner: SystemProgram.programId,
          executable: false,
        },
      })),
      {
        address: userAtaXPda,
        info: {
          data: userAtaXPubkeyData,
          executable: false,
          lamports: LAMPORTS_PER_SOL,
          owner: TOKEN_PROGRAM_ID,
        },
      },
      {
        address: userAtaYPda,
        info: {
          data: userAtaYPubkeyData,
          executable: false,
          lamports: LAMPORTS_PER_SOL,
          owner: TOKEN_PROGRAM_ID,
        },
      },
    ]));

    await program.methods
      .initializeConfig({
        seed,
        locked: false,
        fee: 100,
      })
      .accounts({
        authority: admin.publicKey,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    await program.methods
      .initializeVaults()
      .accountsPartial({
        authority: admin.publicKey,
        config: configPda,
        mintX: mintX.publicKey,
        mintY: mintY.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    const amount = 5;
    const slippage = 0.01;

    await program.methods
      .deposit({
        amount: new BN(amount),
        maxX: new BN(amount * (1 + slippage)),
        maxY: new BN(amount * (1 + slippage)),
      })
      .accountsPartial({
        user: user.publicKey,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
  });

  test("withdraw from a pool", async () => {
    const vaultXPda = getAssociatedTokenAddressSync(
      mintX.publicKey,
      configPda,
      true,
      TOKEN_PROGRAM_ID
    );
    const vaultYPda = getAssociatedTokenAddressSync(
      mintY.publicKey,
      configPda,
      true,
      TOKEN_PROGRAM_ID
    );

    const initVaultXBal = (await getAccount(provider.connection, vaultXPda))
      .amount;
    const initVaultYBal = (await getAccount(provider.connection, vaultYPda))
      .amount;

    const initUserAtaXBal = (await getAccount(provider.connection, userAtaXPda))
      .amount;
    const initUserAtaYBal = (await getAccount(provider.connection, userAtaYPda))
      .amount;

    const [mintLp] = getMintLpPdaAndBump(configPda);
    const userAtaLpPda = getAssociatedTokenAddressSync(
      mintLp,
      user.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const initUserAtaLpBal = (
      await getAccount(provider.connection, userAtaLpPda)
    ).amount;

    const amount = 3;
    const slippage = 0.01;

    await program.methods
      .withdraw({
        amount: new BN(amount),
        minX: new BN(amount * (1 - slippage)),
        minY: new BN(amount * (1 - slippage)),
      })
      .accountsPartial({
        user: user.publicKey,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const postVaultXBal = (await getAccount(provider.connection, vaultXPda))
      .amount;
    const postVaultYBal = (await getAccount(provider.connection, vaultYPda))
      .amount;

    // possible rounding errors from withdraw calculation
    expect(Number(postVaultXBal)).toBeLessThanOrEqual(
      Number(initVaultXBal) - amount
    );
    expect(Number(postVaultYBal)).toBeLessThanOrEqual(
      Number(initVaultYBal) - amount
    );

    const postUserAtaXBal = (await getAccount(provider.connection, userAtaXPda))
      .amount;
    const postUserAtaYBal = (await getAccount(provider.connection, userAtaYPda))
      .amount;

    // possible rounding errors from withdraw calculation
    expect(Number(postUserAtaXBal)).toBeGreaterThanOrEqual(
      Number(initUserAtaXBal) + amount
    );
    expect(Number(postUserAtaYBal)).toBeGreaterThanOrEqual(
      Number(initUserAtaYBal) + amount
    );

    const postUserAtaLpBal = (
      await getAccount(provider.connection, userAtaLpPda)
    ).amount;

    expect(Number(postUserAtaLpBal)).toEqual(Number(initUserAtaLpBal) - amount);
  });

  test("throws if withdrawing from a locked pool", async () => {
    await program.methods
      .update({
        locked: true,
        fee: null,
        authority: null,
      })
      .accountsPartial({
        authority: admin.publicKey,
        config: configPda,
      })
      .signers([admin])
      .rpc();

    const amount = 3;
    const slippage = 0.01;

    try {
      await program.methods
        .withdraw({
          amount: new BN(amount),
          minX: new BN(amount * (1 - slippage)),
          minY: new BN(amount * (1 - slippage)),
        })
        .accountsPartial({
          user: user.publicKey,
          config: configPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      expect(err).toBeInstanceOf(AnchorError);

      const { error } = err as AnchorError;
      expect(error.errorCode.code).toEqual("PoolLocked");
      expect(error.errorCode.number).toEqual(6001);
    }
  });

  test("throws if amount to withdraw is zero", async () => {
    const amount = 0;
    const slippage = 0.01;

    try {
      await program.methods
        .withdraw({
          amount: new BN(amount),
          minX: new BN(amount * (1 - slippage)),
          minY: new BN(amount * (1 - slippage)),
        })
        .accountsPartial({
          user: user.publicKey,
          config: configPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      expect(err).toBeInstanceOf(AnchorError);

      const { error } = err as AnchorError;
      expect(error.errorCode.code).toEqual("InvalidAmount");
      expect(error.errorCode.number).toEqual(6002);
    }
  });
});
