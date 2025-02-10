import { Keypair } from "@solana/web3.js";

export const [mintX, mintY] = Array.from({ length: 2 }, Keypair.generate);
