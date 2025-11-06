//! Solana SPL 代币程序测试

use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::{
    instruction::{initialize_mint, mint_to, transfer},
    state::Mint,
};
use spl_associated_token_account::instruction::create_associated_token_account;

use crate::{
    process_instruction, TokenInstruction, TokenAccountState, validate_token_account,
    get_associated_token_address,
};

const LAMPORTS_PER_TOKEN: u64 = 1_000_000_000;

/// 设置测试程序环境
async fn setup_program_test() -> (ProgramTestContext, Keypair, Keypair, Keypair) {
    let program_id = Pubkey::new_unique();

    let mut program_test = ProgramTest::new(
        "solana_spl_token",
        program_id,
        processor!(process_instruction),
    );

    // 添加 SPL Token 程序
    program_test.add_program(
        "spl_token",
        spl_token::id(),
        processor!(spl_token::processor::process_instruction),
    );

    // 添加关联代币账户程序
    program_test.add_program(
        "spl_associated_token_account",
        spl_associated_token_account::id(),
        processor!(spl_associated_token_account::processor::process_instruction),
    );

    let context = program_test.start().await;

    // 创建测试密钥对
    let mint_authority = Keypair::new();
    let user = Keypair::new();
    let recipient = Keypair::new();

    // 为账户提供初始余额
    for keypair in [&context.payer, &mint_authority, &user, &recipient] {
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[solana_sdk::system_instruction::transfer(
                    &context.payer.pubkey(),
                    &keypair.pubkey(),
                    10 * LAMPORTS_PER_TOKEN,
                )],
                Some(&context.payer.pubkey()),
                &[&context.payer],
                context.last_blockhash,
            ))
            .await
            .unwrap();
    }

    (context, mint_authority, user, recipient)
}

/// 测试代币铸造账户初始化
#[tokio::test]
async fn test_initialize_mint() {
    let (mut context, mint_authority, _user, _recipient) = setup_program_test().await;

    // 创建代币铸造账户
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();

    // 构建初始化指令
    let decimals = 9;
    let freeze_authority: Option<Pubkey> = None;

    let init_mint_ix = TokenInstruction::InitializeMint {
        decimals,
        mint_authority: mint_authority.pubkey(),
        freeze_authority,
    };

    let instruction = Instruction::new_with_bincode(
        &crate::id(),
        &init_mint_ix,
        vec![
            AccountMeta::new(mint_pubkey, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        ],
    );

    // 执行交易
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let result = context.banks_client.process_transaction(transaction).await;

    // 验证交易结果
    assert!(result.is_ok(), "初始化代币铸造账户失败");

    // 验证铸造账户状态
    let mint_account = context
        .banks_client
        .get_account(mint_pubkey)
        .await
        .unwrap()
        .unwrap();

    let mint_data = Mint::unpack(&mint_account.data).unwrap();
    assert_eq!(mint_data.decimals, decimals);
    assert_eq!(mint_data.mint_authority, Some(mint_authority.pubkey()));
    assert_eq!(mint_data.supply, 0);
    assert!(!mint_data.is_initialized);

    println!("✅ 代币铸造账户初始化测试通过");
}

/// 测试代币账户创建
#[tokio::test]
async fn test_create_token_account() {
    let (mut context, _mint_authority, user, _recipient) = setup_program_test().await;

    // 创建代币铸造账户
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();

    // 首先初始化铸造账户
    let init_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &context.payer.pubkey(),
        None,
        9,
    );

    // 创建用户的关联代币账户
    let user_token_account = get_associated_token_address(&user.pubkey(), &mint_pubkey);
    let create_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &user.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    // 构建交易
    let transaction = Transaction::new_signed_with_payer(
        &[init_mint_ix, create_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_keypair],
        context.last_blockhash,
    );

    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "创建代币账户失败");

    // 验证代币账户
    let token_account = context
        .banks_client
        .get_account(user_token_account)
        .await
        .unwrap()
        .unwrap();

    assert!(!token_account.data.is_empty(), "代币账户数据为空");

    println!("✅ 代币账户创建测试通过");
}

/// 测试代币铸造
#[tokio::test]
async fn test_mint_tokens() {
    let (mut context, mint_authority, user, _recipient) = setup_program_test().await;

    // 设置代币铸造账户和用户代币账户
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let user_token_account = get_associated_token_address(&user.pubkey(), &mint_pubkey);

    // 初始化铸造账户
    let init_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority.pubkey(),
        None,
        9,
    );

    // 创建用户代币账户
    let create_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &user.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    // 铸造代币
    let mint_amount = 1000 * LAMPORTS_PER_TOKEN;
    let mint_ix = mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &user_token_account,
        &mint_authority.pubkey(),
        &[],
        mint_amount,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[init_mint_ix, create_account_ix, mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_keypair, &mint_authority],
        context.last_blockhash,
    );

    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "代币铸造失败");

    // 验证代币余额
    let token_balance = context
        .banks_client
        .get_token_account_balance(user_token_account)
        .await
        .unwrap();

    assert_eq!(token_balance.amount, mint_amount.to_string());

    println!("✅ 代币铸造测试通过");
}

/// 测试代币转移
#[tokio::test]
async fn test_transfer_tokens() {
    let (mut context, mint_authority, user, recipient) = setup_program_test().await;

    // 设置代币铸造账户和用户代币账户
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let user_token_account = get_associated_token_address(&user.pubkey(), &mint_pubkey);
    let recipient_token_account = get_associated_token_address(&recipient.pubkey(), &mint_pubkey);

    // 初始化铸造账户
    let init_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority.pubkey(),
        None,
        9,
    );

    // 创建用户代币账户
    let create_user_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &user.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    // 创建接收者代币账户
    let create_recipient_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &recipient.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    // 给用户铸造代币
    let mint_amount = 1000 * LAMPORTS_PER_TOKEN;
    let mint_ix = mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &user_token_account,
        &mint_authority.pubkey(),
        &[],
        mint_amount,
    );

    // 转移代币
    let transfer_amount = 100 * LAMPORTS_PER_TOKEN;
    let transfer_ix = transfer(
        &spl_token::id(),
        &user_token_account,
        &recipient_token_account,
        &user.pubkey(),
        &[],
        transfer_amount,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[
            init_mint_ix,
            create_user_account_ix,
            create_recipient_account_ix,
            mint_ix,
            transfer_ix,
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_keypair, &mint_authority, &user],
        context.last_blockhash,
    );

    let result = context.banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "代币转移失败");

    // 验证转移后的余额
    let user_balance = context
        .banks_client
        .get_token_account_balance(user_token_account)
        .await
        .unwrap();

    let recipient_balance = context
        .banks_client
        .get_token_account_balance(recipient_token_account)
        .await
        .unwrap();

    assert_eq!(user_balance.amount, (mint_amount - transfer_amount).to_string());
    assert_eq!(recipient_balance.amount, transfer_amount.to_string());

    println!("✅ 代币转移测试通过");
}

/// 测试代币账户验证功能
#[tokio::test]
async fn test_validate_token_account() {
    let (mut context, _mint_authority, user, _recipient) = setup_program_test().await;

    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let user_token_account = get_associated_token_address(&user.pubkey(), &mint_pubkey);

    // 创建并初始化账户
    let init_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &context.payer.pubkey(),
        None,
        9,
    );

    let create_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &user.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[init_mint_ix, create_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_keypair],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(transaction).await.unwrap();

    // 获取代币账户信息
    let account_info = context
        .banks_client
        .get_account(user_token_account)
        .await
        .unwrap()
        .unwrap();

    // 创建 AccountInfo（这里简化处理）
    let account_info = AccountInfo::new(
        &user_token_account,
        false,
        true,
        &mut account_info.lamports,
        &mut account_info.data,
        &account_info.owner,
        account_info.executable,
        account_info.rent_epoch,
    );

    // 验证代币账户
    let result = validate_token_account(&account_info, &mint_pubkey, &user.pubkey());
    assert!(result.is_ok(), "代币账户验证失败");

    // 测试无效的铸造账户
    let wrong_mint = Pubkey::new_unique();
    let result = validate_token_account(&account_info, &wrong_mint, &user.pubkey());
    assert!(result.is_err(), "应该验证失败");

    println!("✅ 代币账户验证测试通过");
}

/// 测试指令序列化和反序列化
#[test]
fn test_instruction_serialization() {
    // 测试初始化指令
    let init_instruction = TokenInstruction::InitializeMint {
        decimals: 9,
        mint_authority: Pubkey::new_unique(),
        freeze_authority: Some(Pubkey::new_unique()),
    };

    let serialized = init_instruction.try_to_vec().unwrap();
    let deserialized = TokenInstruction::try_from_slice(&serialized).unwrap();

    match deserialized {
        TokenInstruction::InitializeMint { decimals, mint_authority, freeze_authority } => {
            assert_eq!(decimals, 9);
            assert!(freeze_authority.is_some());
        }
        _ => panic!("指令序列化失败"),
    }

    // 测试铸造指令
    let mint_instruction = TokenInstruction::MintTokens { amount: 1000 };
    let serialized = mint_instruction.try_to_vec().unwrap();
    let deserialized = TokenInstruction::try_from_slice(&serialized).unwrap();

    match deserialized {
        TokenInstruction::MintTokens { amount } => {
            assert_eq!(amount, 1000);
        }
        _ => panic!("指令序列化失败"),
    }

    println!("✅ 指令序列化测试通过");
}

/// 测试关联代币账户地址生成
#[test]
fn test_associated_token_address() {
    let wallet = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let address1 = get_associated_token_address(&wallet, &mint);
    let address2 = get_associated_token_address(&wallet, &mint);

    // 确保相同参数生成相同地址
    assert_eq!(address1, address2);

    // 确保不同参数生成不同地址
    let different_wallet = Pubkey::new_unique();
    let address3 = get_associated_token_address(&different_wallet, &mint);
    assert_ne!(address1, address3);

    let different_mint = Pubkey::new_unique();
    let address4 = get_associated_token_address(&wallet, &different_mint);
    assert_ne!(address1, address4);

    println!("✅ 关联代币账户地址生成测试通过");
}

/// 性能测试：大量代币转移
#[tokio::test]
async fn test_bulk_transfers() {
    let (mut context, mint_authority, user, recipient) = setup_program_test().await;

    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let user_token_account = get_associated_token_address(&user.pubkey(), &mint_pubkey);
    let recipient_token_account = get_associated_token_address(&recipient.pubkey(), &mint_pubkey);

    // 设置账户
    let init_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority.pubkey(),
        None,
        9,
    );

    let create_user_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &user.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    let create_recipient_account_ix = create_associated_token_account(
        &context.payer.pubkey(),
        &recipient.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );

    let mint_amount = 10000 * LAMPORTS_PER_TOKEN;
    let mint_ix = mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &user_token_account,
        &mint_authority.pubkey(),
        &[],
        mint_amount,
    );

    let setup_transaction = Transaction::new_signed_with_payer(
        &[
            init_mint_ix,
            create_user_account_ix,
            create_recipient_account_ix,
            mint_ix,
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_keypair, &mint_authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(setup_transaction).await.unwrap();

    // 执行多次转移
    let transfer_count = 10;
    let transfer_amount = 10 * LAMPORTS_PER_TOKEN;

    for i in 0..transfer_count {
        let transfer_ix = transfer(
            &spl_token::id(),
            &user_token_account,
            &recipient_token_account,
            &user.pubkey(),
            &[],
            transfer_amount,
        );

        let transaction = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &user],
            context.last_blockhash,
        );

        let result = context.banks_client.process_transaction(transaction).await;
        assert!(result.is_ok(), "第 {} 次转移失败", i + 1);
    }

    // 验证最终余额
    let final_user_balance = context
        .banks_client
        .get_token_account_balance(user_token_account)
        .await
        .unwrap();

    let final_recipient_balance = context
        .banks_client
        .get_token_account_balance(recipient_token_account)
        .await
        .unwrap();

    let expected_user_balance = mint_amount - (transfer_count * transfer_amount);
    let expected_recipient_balance = transfer_count * transfer_amount;

    assert_eq!(final_user_balance.amount, expected_user_balance.to_string());
    assert_eq!(final_recipient_balance.amount, expected_recipient_balance.to_string());

    println!("✅ 批量代币转移测试通过");
}