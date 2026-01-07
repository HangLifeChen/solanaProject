import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Voting } from "../target/types/voting";
import { assert, expect } from "chai";

describe("Voting", () => {
  // 1. 配置客户端连接（这里用 Anchor 默认的本地或 Devnet 配置）
  anchor.setProvider(anchor.AnchorProvider.env());

  // 2. 获取程序对象（根据 idl 自动生成）
  const program = anchor.workspace.voting as Program<Voting>;

  // 3. 获取 provider（里面有 wallet、公钥、连接信息）
  const provider = anchor.getProvider();
  const wallet = provider.wallet as anchor.Wallet;

  // 4. 定义测试用到的变量
  const pollId = new anchor.BN(17); // 投票 ID
  const pollDescription = "Test Poll"; // 投票描述
  const pollStart = new anchor.BN(Date.now()); // 投票开始时间
  const pollEnd = new anchor.BN(Date.now() + 1000 * 60 * 60); // 投票结束时间（1小时后）

  const candidateName = "Alice"; // 候选人名字

  // 5. 计算 PDA（投票账户） // [publicKey,  // PDA（PublicKey 对象）bump  bump 值（u8）] 解构数组只关心第一个值
  const [pollPda] = anchor.web3.PublicKey.findProgramAddressSync( //
    [pollId.toArrayLike(Buffer, "le", 8)], // seeds 必须和 Rust 里一致
    program.programId
  );

  // 6. 计算 PDA（候选人账户）
  const [candidatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      pollId.toArrayLike(Buffer, "le", 8),
      Buffer.from(anchor.utils.bytes.utf8.encode(candidateName))
    ],
    program.programId
  );

  it("initializes a poll", async () => {
    // 调用 Rust 里的 initialize_poll 方法
    const tx = await program.methods
      .initializePoll(pollId, pollDescription, pollStart, pollEnd)
      .accounts({
        // 投票账户 PDA
        // poll: "aaaa",
        signer: wallet.publicKey,  // 付款人 + 签名人
      })
      .rpc();

    console.log("✅ Poll initialized, tx signature:", tx);

    // 从链上读取 poll 账户数据
    const pollAccount = await program.account.poll.fetch(pollPda);
    console.log("📋 Poll account data:", pollAccount);
  });

  it("initializes a candidate", async () => {
    const tx = await program.methods
      .initializeCandidate(pollId, candidateName)
      .accounts({
        signer: wallet.publicKey,
      })
      .rpc();

    console.log("✅ Candidate initialized, tx signature:", tx);

    const candidateAccount = await program.account.candidate.fetch(candidatePda);
    console.log("📋 Candidate account data:", candidateAccount);
    assert.equal(candidateAccount.name, candidateName);
    assert.equal(candidateAccount.voteCount.toString(), "0");
    expect(candidateAccount.name).to.equal(candidateName);
    expect(candidateAccount.voteCount.toString()).to.equal("0");
  });
  it("votes for a candidate", async () => {
    const tx = await program.methods
      .vote(pollId, candidateName)
      .accounts({
        signer: wallet.publicKey,
      })
      .rpc();

    console.log("✅ Voted for candidate, tx signature:", tx);

    const candidateAccount = await program.account.candidate.fetch(candidatePda);
    console.log("📋 Candidate account data:", candidateAccount);
    assert.equal(candidateAccount.name, candidateName);
    assert.equal(candidateAccount.voteCount.toString(), "1");
    expect(candidateAccount.name).to.equal(candidateName);
    expect(candidateAccount.voteCount.toString()).to.equal("1");
  });
});
