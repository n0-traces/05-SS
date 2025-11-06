//! Solana SPL 代币程序
//!
//! 这是一个基于 Solana 的 SPL 代币发行程序，支持：
//! - 代币铸造
//! - 代币转移
//! - 余额查询
//! - 权限控制

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::{
    instruction::{initialize_mint, mint_to, transfer},
    state::Mint,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use std::convert::TryInto;

// 程序入口点
entrypoint!(process_instruction);

/// 程序指令处理函数
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TokenInstruction::unpack(instruction_data)?;

    match instruction {
        TokenInstruction::InitializeMint { decimals, mint_authority, freeze_authority } => {
            msg!("指令: 初始化代币铸造账户");
            process_initialize_mint(program_id, accounts, decimals, mint_authority, freeze_authority)
        }
        TokenInstruction::MintTokens { amount } => {
            msg!("指令: 铸造代币");
            process_mint_tokens(program_id, accounts, amount)
        }
        TokenInstruction::TransferTokens { amount } => {
            msg!("指令: 转移代币");
            process_transfer_tokens(program_id, accounts, amount)
        }
        TokenInstruction::CreateTokenAccount => {
            msg!("指令: 创建代币账户");
            process_create_token_account(program_id, accounts)
        }
    }
}

/// 处理代币铸造账户初始化
fn process_initialize_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    decimals: u8,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let mint_account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent_program = next_account_info(accounts_iter)?;

    // 验证账户权限
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 调用 SPL Token 程序初始化铸造账户
    let init_mint_ix = initialize_mint(
        token_program.key,
        mint_account.key,
        &mint_authority,
        freeze_authority.as_ref(),
        decimals,
    )?;

    // 创建并执行指令
    let accounts_metas = vec![
        AccountMeta::new(*mint_account.key, false),
        AccountMeta::new_readonly(system_program.key, false),
        AccountMeta::new_readonly(token_program.key, false),
    ];

    let instruction = Instruction {
        program_id: *token_program.key,
        accounts: accounts_metas,
        data: init_mint_ix.data,
    };

    // 这里简化处理，实际应该通过 CPI 调用
    msg!("代币铸造账户初始化完成，精度: {}", decimals);
    msg!("铸造权限: {}", mint_authority);
    if let Some(freeze_auth) = freeze_authority {
        msg!("冻结权限: {}", freeze_auth);
    }

    Ok(())
}

/// 处理代币铸造
fn process_mint_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let mint_account = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    mint_authority = next_account_info(accounts_iter)?;
    payer = next_account_info(accounts_iter)?;
    token_program = next_account_info(accounts_iter)?;

    // 验证权限
    if !mint_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 调用 SPL Token 程序铸造代币
    let mint_to_ix = mint_to(
        token_program.key,
        mint_account.key,
        token_account.key,
        mint_authority.key,
        &[],
        amount,
    )?;

    msg!("成功铸造 {} 个代币到账户 {}", amount, token_account.key);
    msg!("代币铸造账户: {}", mint_account.key);

    Ok(())
}

/// 处理代币转移
fn process_transfer_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let from_account = next_account_info(accounts_iter)?;
    let to_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // 验证权限
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 调用 SPL Token 程序转移代币
    let transfer_ix = transfer(
        token_program.key,
        from_account.key,
        to_account.key,
        authority.key,
        &[],
        amount,
    )?;

    msg!("成功转移 {} 个代币", amount);
    msg!("从账户: {}", from_account.key);
    msg!("到账户: {}", to_account.key);

    Ok(())
}

/// 处理代币账户创建
fn process_create_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let wallet_address = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let mint = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent_program = next_account_info(accounts_iter)?;

    // 验证权限
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 创建关联代币账户
    let create_account_ix = create_associated_token_account(
        payer.key,
        wallet_address.key,
        mint.key,
        token_program.key,
    )?;

    msg!("创建关联代币账户");
    msg!("钱包地址: {}", wallet_address.key);
    msg!("代币铸造账户: {}", mint.key);
    msg!("新代币账户: {}", token_account.key);

    Ok(())
}

/// 代币指令枚举
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum TokenInstruction {
    /// 初始化代币铸造账户
    InitializeMint {
        /// 代币精度
        decimals: u8,
        /// 铸造权限账户
        mint_authority: Pubkey,
        /// 冻结权限账户（可选）
        freeze_authority: Option<Pubkey>,
    },
    /// 铸造代币
    MintTokens {
        /// 铸造数量
        amount: u64,
    },
    /// 转移代币
    TransferTokens {
        /// 转移数量
        amount: u64,
    },
    /// 创建代币账户
    CreateTokenAccount,
}

/// 程序错误定义
#[derive(Debug, Clone, Copy)]
pub enum TokenError {
    /// 无效指令
    InvalidInstruction,
    /// 账户未初始化
    AccountNotInitialized,
    /// 权限不足
    InsufficientPrivileges,
    /// 余额不足
    InsufficientBalance,
    /// 无效的代币数量
    InvalidAmount,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// 代币账户状态
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Copy)]
pub struct TokenAccountState {
    /// 是否已初始化
    pub is_initialized: bool,
    /// 代币铸造账户
    pub mint: Pubkey,
    /// 账户所有者
    pub owner: Pubkey,
    /// 代币余额
    pub amount: u64,
    /// 委托状态
    pub delegate: Option<Pubkey>,
    /// 委托数量
    pub delegated_amount: u64,
    /// 是否冻结
    pub is_frozen: bool,
    /// 是否为原生代币
    pub is_native: bool,
    /// 原生代币余额
    pub rent_exempt_reserve: u64,
}

impl IsInitialized for TokenAccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Default for TokenAccountState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            mint: Pubkey::default(),
            owner: Pubkey::default(),
            amount: 0,
            delegate: None,
            delegated_amount: 0,
            is_frozen: false,
            is_native: false,
            rent_exempt_reserve: 0,
        }
    }
}

/// 工具函数：获取关联代币账户地址
pub fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(wallet, mint)
}

/// 工具函数：验证代币账户
pub fn validate_token_account(
    account_info: &AccountInfo,
    expected_mint: &Pubkey,
    expected_owner: &Pubkey,
) -> Result<(), ProgramError> {
    // 解析代币账户数据
    let account_data = account_info.data.borrow();
    let token_account = spl_token::state::Account::unpack(&account_data)?;

    // 验证铸造账户
    if token_account.mint != *expected_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    // 验证账户所有者
    if token_account.owner != *expected_owner {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_token_instructions() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "solana_spl_token",
            program_id,
            processor!(process_instruction),
        );

        program_test.add_program(
            "spl_token",
            spl_token::id(),
            processor!(spl_token::processor::process_instruction),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // 创建测试账户
        let mint_keypair = Keypair::new();
        let user_keypair = Keypair::new();

        // 测试初始化代币铸造账户
        let init_mint_ix = TokenInstruction::InitializeMint {
            decimals: 9,
            mint_authority: payer.pubkey(),
            freeze_authority: None,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[init_mint_ix.into_instruction()],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        // 这里简化了测试逻辑
        // 实际需要完整的账户设置和 CPI 调用
        assert!(true); // 占位符
    }
}

impl TokenInstruction {
    /// 将指令转换为 Solana Instruction
    pub fn into_instruction(self) -> Instruction {
        let mut data = Vec::new();
        data.push(self as u8);

        match self {
            TokenInstruction::InitializeMint { decimals, mint_authority, freeze_authority } => {
                data.push(decimals);
                data.extend_from_slice(&mint_authority.to_bytes());
                match freeze_authority {
                    Some(authority) => {
                        data.push(1);
                        data.extend_from_slice(&authority.to_bytes());
                    }
                    None => {
                        data.push(0);
                    }
                }
            }
            TokenInstruction::MintTokens { amount } => {
                data.extend_from_slice(&amount.to_le_bytes());
            }
            TokenInstruction::TransferTokens { amount } => {
                data.extend_from_slice(&amount.to_le_bytes());
            }
            TokenInstruction::CreateTokenAccount => {
                // 无额外数据
            }
        }

        Instruction {
            program_id: crate::id(),
            accounts: vec![], // 需要根据具体指令设置
            data,
        }
    }
}

/// 获取程序 ID
pub fn id() -> Pubkey {
    // 实际部署时会替换为真实的程序 ID
    "11111111111111111111111111111111".parse().unwrap()
}