# Solana SPL 代币程序实现详细指南

## 环境准备

### 1. 系统要求
- 操作系统：Linux/macOS/Windows (推荐 Linux)
- 内存：至少 8GB RAM
- 磁盘空间：至少 10GB 可用空间

### 2. 工具安装详解

#### Rust 工具链
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version

# 添加 WASM 目标（智能合约需要）
rustup target add wasm32-unknown-unknown
rustup target add bpfel-unknown-unknown
```

#### Solana CLI
```bash
# 安装 Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"

# 添加到环境变量
export PATH="/home/tdp/.local/share/solana/install/active_release/bin:$PATH"
echo 'export PATH="/home/tdp/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc

# 配置网络
solana config set --url devnet
solana config get

# 创建新钱包（如果还没有）
solana-keygen new --outfile ~/.config/solana/id.json

# 获取测试网 SOL
solana airdrop 2
```

#### Anchor 框架
```bash
# 安装 Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# 验证安装
anchor --version
```

## 项目创建和结构

### 1. 初始化 Anchor 项目
```bash
cd /home/tdp/contract_projects/task04
anchor init solana-spl-token --no-git
cd solana-spl-token
```

### 2. 项目结构详解
```
solana-spl-token/
├── .anchor/                     # Anchor 配置
├── app/                         # 前端应用（可选）
├── migrations/                  # 部署脚本
├── programs/                    # 智能合约程序
│   └── solana-spl-token/
│       ├── Cargo.toml          # Rust 项目配置
│       └── src/
│           └── lib.rs          # 主要逻辑
├── tests/                      # 测试文件
│   └── solana-spl-token.ts     # TypeScript 测试
├── Anchor.toml                 # Anchor 项目配置
├── Cargo.toml                  # 工作空间配置
├── package.json                # Node.js 配置
├── tsconfig.json              # TypeScript 配置
└── yarn.lock                  # 依赖锁定文件
```

## SPL 代币程序实现

### 1. 依赖配置

首先更新 `programs/solana-spl-token/Cargo.toml`：
```toml
[dependencies]
anchor-lang = "0.30.1"
anchor-spl = "0.30.1"
spl-token = "4.0.0"
spl-associated-token-account = "2.0.0"
```

### 2. 主程序实现

编辑 `programs/solana-spl-token/src/lib.rs`：

```rust
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer, MintTo, Burn},
    associated_token::AssociatedToken,
};

declare_id!("11111111111111111111111111111111"); // 将在部署后替换为实际程序 ID

#[program]
pub mod solana_spl_token {
    use super::*;

    /// 创建新的代币铸造账户
    pub fn create_token_mint(
        ctx: Context<CreateTokenMint>,
        decimals: u8,
    ) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // 初始化铸造账户
        anchor_spl::token::initialize_mint(
            ctx.accounts.mint.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.authority.to_account_info(),
            None, // 冻结权限
            decimals,
        )?;

        msg!("代币铸造账户创建成功，精度: {}", decimals);
        Ok(())
    }

    /// 铸造代币到指定账户
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        let seeds = &[
            b"mint_authority",
            &[ctx.bumps.mint_authority],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        anchor_spl::token::mint_to(cpi_ctx, amount)?;

        msg!("成功铸造 {} 个代币到账户 {}", amount, ctx.accounts.recipient.key());
        Ok(())
    }

    /// 转移代币
    pub fn transfer_tokens(
        ctx: Context<TransferTokens>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.to_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        msg!("成功转移 {} 个代币从 {} 到 {}",
             amount,
             ctx.accounts.from_token_account.key(),
             ctx.accounts.to_token_account.key());
        Ok(())
    }

    /// 销毁代币
    pub fn burn_tokens(
        ctx: Context<BurnTokens>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        anchor_spl::token::burn(cpi_ctx, amount)?;

        msg!("成功销毁 {} 个代币", amount);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreateTokenMint<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = authority,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: 铸造权限账户
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: 接收者账户
    #[account()]
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: 铸造权限账户
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
```

### 3. 测试文件实现

更新 `tests/solana-spl-token.ts`：

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaSplToken } from "../target/types/solana_spl_token";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction
} from "@solana/spl-token";
import {
  LAMPORTS_PER_SOL,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY
} from "@solana/web3.js";

describe("solana-spl-token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaSplToken as Program<SolanaSplToken>;

  // 测试用的密钥对
  const mint = Keypair.generate();
  const recipient = Keypair.generate();

  // 铸造权限 PDA
  let mintAuthorityPda: PublicKey;
  let mintAuthorityBump: number;

  before(async () => {
    // 计算铸造权限 PDA
    const [mintAuthority, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("mint_authority")],
      program.programId
    );
    mintAuthorityPda = mintAuthority;
    mintAuthorityBump = bump;
  });

  it("创建代币铸造账户", async () => {
    const decimals = 9; // 9 位小数，类似 SOL

    const tx = await program.methods
      .createTokenMint(decimals)
      .accounts({
        mint: mint.publicKey,
        authority: provider.wallet.publicKey,
        payer: provider.wallet.publicKey,
        mintAuthority: mintAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([mint])
      .rpc();

    console.log("创建铸造账户交易哈希:", tx);

    // 验证铸造账户信息
    const mintInfo = await provider.connection.getAccountInfo(mint.publicKey);
    console.log("铸造账户信息:", mintInfo);
  });

  it("铸造代币", async () => {
    const amount = new anchor.BN(1000 * LAMPORTS_PER_SOL); // 铸造 1000 个代币

    // 获取接收者的关联代币账户地址
    const recipientTokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      recipient.publicKey
    );

    const tx = await program.methods
      .mintTokens(amount)
      .accounts({
        mint: mint.publicKey,
        recipient: recipient.publicKey,
        mintAuthority: mintAuthorityPda,
        payer: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([
        createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey,
          recipientTokenAccount,
          recipient.publicKey,
          mint.publicKey
        )
      ])
      .rpc();

    console.log("铸造代币交易哈希:", tx);

    // 检查余额
    const tokenBalance = await provider.connection.getTokenAccountBalance(
      recipientTokenAccount
    );
    console.log("代币余额:", tokenBalance.value.uiAmount);
  });

  it("转移代币", async () => {
    const amount = new anchor.BN(100 * LAMPORTS_PER_SOL); // 转移 100 个代币

    // 创建接收者账户
    const newRecipient = Keypair.generate();
    const newRecipientTokenAccount = await getAssociatedTokenAddress(
      mint.publicKey,
      newRecipient.publicKey
    );

    const tx = await program.methods
      .transferTokens(amount)
      .accounts({
        fromTokenAccount: await getAssociatedTokenAddress(mint.publicKey, recipient.publicKey),
        toTokenAccount: newRecipientTokenAccount,
        authority: recipient.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .preInstructions([
        createAssociatedTokenAccountInstruction(
          provider.wallet.publicKey,
          newRecipientTokenAccount,
          newRecipient.publicKey,
          mint.publicKey
        )
      ])
      .signers([recipient])
      .rpc();

    console.log("转移代币交易哈希:", tx);
  });
});
```

## 部署和运行指南

### 1. 本地测试

```bash
# 启动本地验证器
solana-test-validator

# 在新终端中运行测试
anchor test

# 或者分步运行
anchor build
anchor deploy
```

### 2. Devnet 部署

```bash
# 切换到 devnet
solana config set --url devnet

# 获取测试 SOL
solana airdrop 5

# 构建程序
anchor build

# 部署到 devnet
anchor deploy --provider.cluster devnet

# 获取程序 ID
solana address -k target/deploy/solana_spl_token-keypair.json
```

### 3. 命令行测试

```bash
# 查看代币供应量
spl-token supply <TOKEN_MINT_ADDRESS>

# 查看账户余额
spl-token accounts

# 转移代币
spl-token transfer <TOKEN_MINT_ADDRESS> <RECIPIENT_ADDRESS> <AMOUNT> --owner <OWNER_KEYPAIR>

# 创建代币账户
spl-token create-account <TOKEN_MINT_ADDRESS> --owner <OWNER_ADDRESS>
```

## 常见问题和解决方案

### 1. 编译错误
```bash
# 清理缓存重新构建
anchor clean
anchor build
```

### 2. 权限问题
```bash
# 检查钱包余额
solana balance

# 检查程序权限
solana program show <PROGRAM_ID>
```

### 3. 网络问题
```bash
# 检查网络连接
solana cluster-version

# 切换网络
solana config set --url localhost  # 本地网络
solana config set --url devnet      # 开发网络
solana config set --url mainnet    # 主网
```

## 性能优化建议

1. **批量操作**：将多个操作合并到单个交易中
2. **PDA 优化**：合理设计程序派生地址的种子
3. **账户管理**：复用现有账户，减少初始化成本
4. **指令优化**：减少不必要的计算和存储操作

## 安全考虑

1. **权限控制**：确保只有授权账户可以执行关键操作
2. **输入验证**：验证所有输入参数的有效性
3. **溢出保护**：使用 Anchor 的内置溢出保护
4. **重放攻击防护**：使用唯一标识符防止重放攻击

## 监控和日志

```rust
// 在程序中添加详细的日志
msg!("操作开始: {:?}", operation_type);
msg!("账户余额: {}", balance);
msg!("操作完成: {:?}", result);
```

使用 Solana CLI 查看程序日志：
```bash
solana logs <PROGRAM_ID>
```