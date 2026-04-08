// @ts-nocheck
import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { expect } from "chai";
import { Crowdfunding } from "../target/types/crowdfunding";

/**
 * Runtime (localnet) integration tests.
 *
 * Run with:
 *   anchor test   (starts a local validator and sets env vars automatically)
 */

// ── Helpers ───────────────────────────────────────────────────────────────────

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const getVaultPda = (programId: PublicKey, campaign: PublicKey): PublicKey =>
  PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), campaign.toBuffer()],
    programId
  )[0];

const getContributionPda = (
  programId: PublicKey,
  campaign: PublicKey,
  donor: PublicKey
): PublicKey =>
  PublicKey.findProgramAddressSync(
    [Buffer.from("contribution"), campaign.toBuffer(), donor.toBuffer()],
    programId
  )[0];

const airdropTo = async (
  connection: anchor.web3.Connection,
  pubkey: PublicKey,
  sol = 10
) => {
  const sig = await connection.requestAirdrop(pubkey, sol * LAMPORTS_PER_SOL);
  await connection.confirmTransaction(sig, "confirmed");
};

// ── Suite ─────────────────────────────────────────────────────────────────────

describe("crowdfunding", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Crowdfunding as Program<Crowdfunding>;
  const { connection } = provider;

  // ── Success path ────────────────────────────────────────────────────────────
  //
  //   create_campaign → contribute → withdraw (before deadline) → FAIL
  //                  → withdraw (after deadline)  → OK
  //                  → withdraw (again)           → FAIL AlreadyClaimed
  //
  it("success path: contribute → withdraw before deadline fails → withdraw after deadline succeeds → double withdraw fails", async function () {
    this.timeout(120_000);

    const creator      = Keypair.generate();
    const donor        = Keypair.generate();
    const campaignKp   = Keypair.generate();

    await airdropTo(connection, creator.publicKey);
    await airdropTo(connection, donor.publicKey);

    const campaign     = campaignKp.publicKey;
    const vault        = getVaultPda(program.programId, campaign);
    const contribution = getContributionPda(program.programId, campaign, donor.publicKey);

    // goal = 1 SOL, deadline = now + 2 s
    const goal     = new BN(1 * LAMPORTS_PER_SOL);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 2);

    // create_campaign ──────────────────────────────────────────────────────────
    const createSig = await program.methods
      .createCampaign(goal, deadline)
      .accounts({
        creator:       creator.publicKey,
        campaign,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator, campaignKp])
      .rpc();

    // contribute ───────────────────────────────────────────────────────────────
    const contributeSig = await program.methods
      .contribute(new BN(2 * LAMPORTS_PER_SOL))
      .accounts({
        donor:         donor.publicKey,
        campaign,
        vault,
        contribution,
        systemProgram: SystemProgram.programId,
      })
      .signers([donor])
      .rpc();

    // withdraw BEFORE deadline → DeadlineNotReached ────────────────────────────
    try {
      await program.methods
        .withdraw()
        .accounts({
          creator:       creator.publicKey,
          campaign,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();
      expect.fail("withdraw before deadline should have thrown");
    } catch (err: any) {
      expect(err.message).to.match(/DeadlineNotReached/i);
    }

    await sleep(2_500); // wait for deadline to pass

    // withdraw AFTER deadline → success ────────────────────────────────────────
    const withdrawSig = await program.methods
      .withdraw()
      .accounts({
        creator:       creator.publicKey,
        campaign,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator])
      .rpc();

    // withdraw AGAIN → AlreadyClaimed ──────────────────────────────────────────
    try {
      await program.methods
        .withdraw()
        .accounts({
          creator:       creator.publicKey,
          campaign,
          vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([creator])
        .rpc();
      expect.fail("double withdraw should have thrown");
    } catch (err: any) {
      expect(err.message).to.match(/AlreadyClaimed/i);
    }

    console.log(JSON.stringify({ createSig, contributeSig, withdrawSig }, null, 2));
  });

  // ── Refund path ─────────────────────────────────────────────────────────────
  //
  //   create_campaign → contribute → refund (after deadline, goal not met) → OK
  //                  → refund (again) → FAIL (account closed)
  //
  it("refund path: contribute → refund after deadline succeeds → double refund fails", async function () {
    this.timeout(120_000);

    const creator    = Keypair.generate();
    const donor      = Keypair.generate();
    const campaignKp = Keypair.generate();

    await airdropTo(connection, creator.publicKey);
    await airdropTo(connection, donor.publicKey);

    const campaign     = campaignKp.publicKey;
    const vault        = getVaultPda(program.programId, campaign);
    const contribution = getContributionPda(program.programId, campaign, donor.publicKey);

    // goal = 100 SOL (will not be met), deadline = now + 2 s
    const goal     = new BN(100 * LAMPORTS_PER_SOL);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 2);

    // create_campaign ──────────────────────────────────────────────────────────
    const createSig = await program.methods
      .createCampaign(goal, deadline)
      .accounts({
        creator:       creator.publicKey,
        campaign,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator, campaignKp])
      .rpc();

    // contribute ───────────────────────────────────────────────────────────────
    const contributeSig = await program.methods
      .contribute(new BN(1 * LAMPORTS_PER_SOL))
      .accounts({
        donor:         donor.publicKey,
        campaign,
        vault,
        contribution,
        systemProgram: SystemProgram.programId,
      })
      .signers([donor])
      .rpc();

    await sleep(2_500); // wait for deadline to pass

    // refund → success (goal not met) ──────────────────────────────────────────
    const refundSig = await program.methods
      .refund()
      .accounts({
        donor:         donor.publicKey,
        campaign,
        vault,
        contribution,
        systemProgram: SystemProgram.programId,
      })
      .signers([donor])
      .rpc();

    // double refund → fail (Contribution account was closed by `close = donor`) ─
    try {
      await program.methods
        .refund()
        .accounts({
          donor:         donor.publicKey,
          campaign,
          vault,
          contribution,
          systemProgram: SystemProgram.programId,
        })
        .signers([donor])
        .rpc();
      expect.fail("double refund should have thrown");
    } catch (err: any) {
      // Anchor throws an account-not-found error because the Contribution PDA
      // was closed (zeroed + rent returned to donor) during the first refund.
      expect(err.message).to.match(/AccountNotInitialized|account|contribut/i);
    }

    console.log(JSON.stringify({ createSig, contributeSig, refundSig }, null, 2));
  });
});