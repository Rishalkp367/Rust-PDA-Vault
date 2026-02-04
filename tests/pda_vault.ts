import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PdaVault } from "../target/types/pda_vault";
import { SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("pda_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PdaVault as Program<PdaVault>;
  const user = provider.wallet;

  const [vaultState] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_state")],
    program.programId
  );

  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    program.programId
  );

  const [userState] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user_state"), user.publicKey.toBuffer()],
    program.programId
  );

  it("Initialize vault", async () => {
    await program.methods.initializeVault().accountsStrict({
      vaultState,
      vault,
      admin: user.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();
  });

  it("Initialize user", async () => {
    await program.methods.initializeUser().accountsStrict({
      userState,
      user: user.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();
  });

  it("Deposit + Withdraw", async () => {
    await program.methods.deposit(new anchor.BN(1_000_000)).accountsStrict({
      vaultState,
      vault,
      userState,
      user: user.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();

    let state = await program.account.userState.fetch(userState);
    assert.equal(state.deposited.toNumber(), 1_000_000);

    await program.methods.withdraw(new anchor.BN(500_000)).accountsStrict({
      vaultState,
      vault,
      userState,
      user: user.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();

    state = await program.account.userState.fetch(userState);
    assert.equal(state.deposited.toNumber(), 500_000);
  });
});
