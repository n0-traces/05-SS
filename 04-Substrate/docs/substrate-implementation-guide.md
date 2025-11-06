# Substrate 区块链开发详细指南

## 环境准备

### 1. 系统要求
- 操作系统：Linux (推荐 Ubuntu 20.04+) 或 macOS
- 内存：至少 16GB RAM
- 磁盘空间：至少 50GB 可用空间
- CPU：多核处理器（建议 4 核以上）

### 2. 开发工具安装

#### Rust 工具链配置
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 添加 WASM 目标
rustup target add wasm32-unknown-unknown

# 安装特定版本（确保兼容性）
rustup install nightly-2023-11-15
rustup default nightly-2023-11-15

# 验证安装
rustc --version
cargo --version
```

#### Substrate 工具链
```bash
# 安装 substrate-contracts-node
cargo install --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.31.0 --force substrate-contracts-node

# 安装 cargo-contract
cargo install --git https://github.com/paritytech/cargo-contract.git --tag v3.2.0 --force cargo-contract

# 验证安装
substrate-contracts-node --version
cargo-contract --version
```

### 3. 系统依赖
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y cmake pkg-config libssl-dev git clang libclang-dev

# macOS
brew install openssl cmake llvm
```

## 项目创建和结构

### 1. 创建节点模板
```bash
cd /home/tdp/contract_projects/task04
git clone https://github.com/substrate-developer-hub/substrate-node-template
cd substrate-node-template

# 创建新的 Git 仓库
git remote remove origin
git init
git add .
git commit -m "Initial commit from substrate-node-template"
```

### 2. 项目结构详解
```
substrate-node-template/
├── node/                       # 节点实现
│   └── src/
│       ├── chain_spec.rs      # 链规范配置
│       ├── command.rs         # CLI 命令处理
│       ├── lib.rs             # 节点库入口
│       ├── main.rs            # 节点二进制入口
│       ├── rpc.rs             # RPC API 配置
│       └── service.rs         # 节点服务实现
├── runtime/                   # Runtime 实现
│   ├── build.rs               # 构建脚本
│   ├── Cargo.toml             # Runtime 配置
│   └── src/
│       ├── lib.rs             # Runtime 库入口
│       ├── pallets/           # 自定义 Pallets
│       │   └── mod.rs         # Pallets 模块
│       └── runtime.rs         # Runtime 配置
├── pallets/                   # 自定义 Pallets 开发目录
│   └── template/              # 模板 Pallet
├── scripts/                   # 构建和部署脚本
├── Cargo.toml                 # 工作空间配置
└── README.md                  # 项目说明
```

## Runtime 基础功能实现

### 1. 账户管理功能

更新 `runtime/src/lib.rs`，确保包含基础账户管理：

```rust
//! Substrate Node Template Runtime
//!
//! 在这里配置和实现运行时逻辑。

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` 宏执行大量代码生成，使得编译时间更长。
#![recursion_limit = "256"]

// 使 pallet 可见。
pub use pallet_balances;
pub use pallet_timestamp;
pub use pallet_transaction_payment;

use frame_support::{
    derive_impl,
    traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, StorageMap},
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        IdentityFee, Weight,
    },
    Parameter, StorageValue,
};
use frame_system::limits::{BlockLength, BlockWeights};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::OpaqueMetadata;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, KeyTypeId, MultiSignature, MultiSigner, OpaqueExtrinsic,
    Perbill, Permill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// 导入 pallets
use frame_support::traits::ConstBool;
use frame_system::EnsureRoot;

// 定义运行时版本信息
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-template"),
    impl_name: create_runtime_str!("node-template"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

// 账户配置
#[cfg(feature = "std")]
pub type NodeBlock = frame_system::mocking::MockBlock<Runtime>;
#[cfg(not(feature = "std"))]
pub type NodeBlock = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
#[cfg(not(feature = "std"))]
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

// 配置运行时
#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive_runtime_call(RuntimeCall)]
    pub enum Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        // 在这里添加自定义 pallets
        Template: pallet_template,
    }

    #[runtime::pallet_index(0)]
    pub type SystemPallet = Pallet<frame_system::Instance1>;

    #[runtime::pallet_index(1)]
    pub type TimestampPallet = Pallet<pallet_timestamp::Instance1>;

    // 系统配置
    #[cfg_attr(feature = "std", derive(frame_support::derive_impl::Debug))]
    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    #[frame_support::pallet]
    pub mod frame_system {
        #[pallet::config]
        pub trait Config: frame_system::Config {
            type BaseCallFilter = frame_support::traits::Everything;
            type BlockWeights = BlockWeights;
            type BlockLength = BlockLength;
            type AccountId = AccountId;
            type RuntimeCall = RuntimeCall;
            type Lookup = AccountIdLookup<AccountId, ()>;
            type Index = Index;
            type BlockNumber = BlockNumber;
            type Hash = Hash;
            type Hashing = BlakeTwo256;
            type Header = generic::Header<BlockNumber, BlakeTwo256>;
            type RuntimeEvent = RuntimeEvent;
            type RuntimeOrigin = RuntimeOrigin;
            type BlockHashCount = ConstU64<250>;
            type DbWeight = RocksDbWeight;
            type Version = RuntimeVersion;
            type PalletInfo = PalletInfo;
            type AccountData = pallet_balances::AccountData<Balance>;
            type OnNewAccount = ();
            type OnKilledAccount = ();
            type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
            type SS58Prefix = ConstU16<42>;
            type OnSetCode = ();
            type MaxConsumers = ConstU32<16>;
        }
    }

    // 余额配置
    #[cfg_attr(feature = "std", derive(frame_support::derive_impl::Debug))]
    #[derive_impl(pallet_balances::config_preludes::SolochainDefaultConfig as pallet_balances::DefaultConfig)]
    #[frame_support::pallet]
    pub mod pallet_balances {
        #[pallet::config]
        pub trait Config: frame_system::Config + Send + Sync {
            type RuntimeEvent = RuntimeEvent;
            type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
            type Balance = Balance;
            type DustRemoval = ();
            type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
            type AccountStore = System;
            type ReserveIdentifier = [u8; 8];
            type FreezeIdentifier = ();
            type MaxLocks = ConstU32<50>;
            type MaxReserves = ConstU32<50>;
            type HoldIdentifier = ();
            type MaxFreezes = ConstU32<0>;
        }
    }

    // 时间戳配置
    #[cfg_attr(feature = "std", derive(frame_support::derive_impl::Debug))]
    #[derive_impl(pallet_timestamp::config_preludes::SolochainDefaultConfig as pallet_timestamp::DefaultConfig)]
    #[frame_support::pallet]
    pub mod pallet_timestamp {
        #[pallet::config]
        pub trait Config: frame_system::Config {
            type Moment = u64;
            type OnTimestampSet = Aura;
            type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
            type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
        }
    }
}

// 常量定义
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub const EPOCH_DURATION_IN_BLOCKS: u64 = 10 * MINUTES;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// 账户和余额常量
pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS;
pub const DOLLARS: Balance = 100 * CENTS;
pub const EXISTENTIAL_DEPOSIT: u128 = 500 * CENTS;

// 创建运行时
construct_runtime!(
    pub enum Runtime
    where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        Template: pallet_template,
    }
);
```

### 2. 交易处理功能

确保在 `node/src/service.rs` 中正确配置交易池：

```rust
use sc_transaction_pool::{error::Error as TxPoolError, BasicPool, FullChainApi};
use sp_transaction_pool::{PoolFuture, TransactionPool, TransactionSource};

pub fn new_full<RuntimeApi, Executor>(
    config: Configuration,
    enable_jsonrpc: bool,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi: Send + Sync + 'static,
    Executor: sc_executor::NativeExecutionDispatch + Send + Sync + 'static,
{
    // ... 其他代码 ...

    let transaction_pool = BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        prometheus_registry,
        task_manager.spawn_handle(),
        client.clone(),
    );

    // ... 其他代码 ...
}
```

## 自定义投票 Pallet 实现

### 1. 创建 Pallet 模板
```bash
# 创建新的 pallet 目录
mkdir -p pallets/voting
cd pallets/voting

# 创建基本文件结构
touch Cargo.toml src/lib.rs src/mock.rs src/tests.rs
```

### 2. Pallet 配置文件 (`pallets/voting/Cargo.toml`)
```toml
[package]
name = "pallet-voting"
version = "4.0.0-dev"
description = "FRAME pallet for voting system."
authors = ["Your Name <your.email@example.com>"]
homepage = "https://substrate.io/"
edition = "2021"
license = "Unlicense"
publish = false
repository = "https://github.com/paritytech/substrate.git"

[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]

[dependencies]
codec = { package = "parity-scale-codec", version = "3.6.1", default-features = false, features = ["derive"] }
scale-info = { version = "2.5.0", default-features = false, features = ["derive"] }
frame-benchmarking = { version = "4.0.0-dev", default-features = false, optional = true }
frame-support = { version = "4.0.0-dev", default-features = false }
frame-system = { version = "4.0.0-dev", default-features = false }
sp-runtime = { version = "24.0.0", default-features = false, features = ["serde"] }
sp-io = { version = "23.0.0", default-features = false }
sp-std = { version = "8.0.0", default-features = false }

[features]
default = ["std"]
std = [
    "codec/std",
    "frame-benchmarking?/std",
    "frame-support/std",
    "frame-system/std",
    "scale-info/std",
    "sp-runtime/std",
    "sp-io/std",
    "sp-std/std",
]
runtime-benchmarks = ["frame-benchmarking/runtime-benchmarks"]
try-runtime = ["frame-support/try-runtime"]
```

### 3. 核心 Pallet 实现 (`pallets/voting/src/lib.rs`)
```rust
//! # Voting Pallet
//!
//! 这个 pallet 提供了一个基础的投票系统，允许用户创建提案并进行投票。

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    traits::Get,
    BoundedVec, Twox128,
};
use frame_system::ensure_signed;
use sp_runtime::{ArithmeticError, DispatchError};
use sp_std::vec::Vec;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// 提案结构体
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Proposal<AccountId, BlockNumber> {
        /// 提案 ID
        pub id: ProposalId,
        /// 提案创建者
        pub proposer: AccountId,
        /// 提案标题
        pub title: BoundedVec<u8, MaxTitleLength>,
        /// 提案描述
        pub description: BoundedVec<u8, MaxDescriptionLength>,
        /// 创建时间
        pub created_at: BlockNumber,
        /// 投票截止时间
        pub voting_deadline: BlockNumber,
        /// 支持票数
        pub yes_votes: u32,
        /// 反对票数
        u32,
        /// 是否已执行
        pub executed: bool,
    }

    /// 投票记录
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Vote<AccountId> {
        /// 投票者
        pub voter: AccountId,
        /// 投票选择 (true = 支持, false = 反对)
        pub choice: bool,
        /// 投票时间
        pub voted_at: BlockNumber,
    }

    /// 提案 ID 类型
    pub type ProposalId = u32;

    /// 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event 类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 提案标题最大长度
        #[pallet::constant]
        type MaxTitleLength: Get<u32>;

        /// 提案描述最大长度
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// 最大活跃提案数量
        #[pallet::constant]
        type MaxActiveProposals: Get<u32>;

        /// 投票通过阈值 (百分比)
        #[pallet::constant]
        type VoteThreshold: Get<u32>;

        /// 提案最小投票持续时间
        #[pallet::constant]
        type MinVotingDuration: Get<Self::BlockNumber>;

        /// 提案最大投票持续时间
        #[pallet::constant]
        type MaxVotingDuration: Get<Self::BlockNumber>;
    }

    // 类型别名
    type MaxTitleLength<T> = <T as Config>::MaxTitleLength;
    type MaxDescriptionLength<T> = <T as Config>::MaxDescriptionLength;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// 存储下一个提案 ID
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, ProposalId, ValueQuery>;

    /// 存储提案
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Twox128,
        ProposalId,
        Proposal<T::AccountId, T::BlockNumber>,
        OptionQuery,
    >;

    /// 存储活跃提案 ID 列表
    #[pallet::storage]
    #[pallet::getter(fn active_proposals)]
    pub type ActiveProposals<T: Config> = StorageValue<
        _,
        BoundedVec<ProposalId, T::MaxActiveProposals>,
        ValueQuery,
    >;

    /// 存储投票记录
    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Twox128,
        ProposalId,
        Twox128,
        T::AccountId,
        Vote<T::AccountId>,
        OptionQuery,
    >;

    /// 存储每个账户的投票记录（防止重复投票）
    #[pallet::storage]
    #[pallet::getter(fn voter_proposals)]
    pub type VoterProposals<T: Config> = StorageMap<
        _,
        Twox128,
        T::AccountId,
        BoundedVec<ProposalId, ConstU32<100>>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 新提案创建
        ProposalCreated {
            proposal_id: ProposalId,
            proposer: T::AccountId,
        },
        /// 投票完成
        Voted {
            proposal_id: ProposalId,
            voter: T::AccountId,
            choice: bool,
        },
        /// 提案执行
        ProposalExecuted {
            proposal_id: ProposalId,
            success: bool,
        },
        /// 提案关闭
        ProposalClosed {
            proposal_id: ProposalId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 提案不存在
        ProposalNotFound,
        /// 提案已存在
        ProposalAlreadyExists,
        /// 提案标题过长
        TitleTooLong,
        /// 提案描述过长
        DescriptionTooLong,
        /// 投票期限已过
        VotingPeriodExpired,
        /// 已经投过票
        AlreadyVoted,
        /// 不是提案创建者
        NotProposer,
        /// 提案已执行
        AlreadyExecuted,
        /// 活跃提案数量已达上限
        TooManyActiveProposals,
        /// 投票持续时间无效
        InvalidVotingDuration,
        /// 算术错误
        ArithmeticError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // 检查并关闭过期的提案
            let proposals_to_close = Self::active_proposals()
                .into_iter()
                .filter(|&id| {
                    if let Some(proposal) = Self::proposals(id) {
                        proposal.voting_deadline <= n
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();

            let mut weight_used = Weight::zero();
            for proposal_id in proposals_to_close {
                weight_used += Self::close_proposal(proposal_id, n);
            }

            weight_used
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新提案
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().writes(3))]
        pub fn create_proposal(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Vec<u8>,
            voting_duration: T::BlockNumber,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;

            // 验证输入
            ensure!(
                title.len() <= MaxTitleLength::<T>::get() as usize,
                Error::<T>::TitleTooLong
            );
            ensure!(
                description.len() <= MaxDescriptionLength::<T>::get() as usize,
                Error::<T>::DescriptionTooLong
            );
            ensure!(
                voting_duration >= T::MinVotingDuration::get()
                    && voting_duration <= T::MaxVotingDuration::get(),
                Error::<T>::InvalidVotingDuration
            );

            // 检查活跃提案数量
            ensure!(
                Self::active_proposals().len() < T::MaxActiveProposals::get() as usize,
                Error::<T>::TooManyActiveProposals
            );

            // 生成新的提案 ID
            let proposal_id = Self::next_proposal_id();
            let new_id = proposal_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            NextProposalId::<T>::put(new_id);

            // 创建提案
            let current_block = frame_system::Pallet::<T>::block_number();
            let voting_deadline = current_block + voting_duration;

            let proposal = Proposal {
                id: proposal_id,
                proposer: proposer.clone(),
                title: BoundedVec::try_from(title)
                    .map_err(|_| Error::<T>::TitleTooLong)?,
                description: BoundedVec::try_from(description)
                    .map_err(|_| Error::<T>::DescriptionTooLong)?,
                created_at: current_block,
                voting_deadline,
                yes_votes: 0,
                no_votes: 0,
                executed: false,
            };

            // 存储提案
            Proposals::<T>::insert(proposal_id, proposal);

            // 添加到活跃提案列表
            ActiveProposals::<T>::try_mutate(|proposals| {
                proposals.try_push(proposal_id)
            }).map_err(|_| Error::<T>::TooManyActiveProposals)?;

            // 触发事件
            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                proposer,
            });

            Ok(())
        }

        /// 对提案投票
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().writes(2))]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
            choice: bool,
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;

            // 检查提案是否存在且未过期
            let mut proposal = Self::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block <= proposal.voting_deadline,
                Error::<T>::VotingPeriodExpired
            );

            ensure!(!proposal.executed, Error::<T>::AlreadyExecuted);

            // 检查是否已经投过票
            ensure!(
                !Votes::<T>::contains_key(proposal_id, &voter),
                Error::<T>::AlreadyVoted
            );

            // 创建投票记录
            let vote = Vote {
                voter: voter.clone(),
                choice,
                voted_at: current_block,
            };

            // 存储投票
            Votes::<T>::insert(proposal_id, &voter, vote);

            // 更新提案的投票计数
            if choice {
                proposal.yes_votes = proposal.yes_votes.checked_add(1)
                    .ok_or(ArithmeticError::Overflow)?;
            } else {
                proposal.no_votes = proposal.no_votes.checked_add(1)
                    .ok_or(ArithmeticError::Overflow)?;
            }

            // 更新提案
            Proposals::<T>::insert(proposal_id, proposal);

            // 记录投票者的提案列表
            VoterProposals::<T>::try_mutate(&voter, |proposals| {
                if !proposals.contains(&proposal_id) {
                    proposals.try_push(proposal_id)
                } else {
                    Ok(())
                }
            }).map_err(|_| DispatchError::Other("Max proposals per voter exceeded"))?;

            // 触发事件
            Self::deposit_event(Event::Voted {
                proposal_id,
                voter,
                choice,
            });

            Ok(())
        }

        /// 执行提案
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            let mut proposal = Self::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(!proposal.executed, Error::<T>::AlreadyExecuted);

            let current_block = frame_system::Pallet::<T>::block_number();

            // 检查投票是否结束
            if current_block <= proposal.voting_deadline {
                return Err(Error::<T>::VotingPeriodExpired.into());
            }

            // 计算投票结果
            let total_votes = proposal.yes_votes + proposal.no_votes;
            let success = if total_votes == 0 {
                false
            } else {
                let yes_percentage = (proposal.yes_votes as u32) * 100 / total_votes;
                yes_percentage >= T::VoteThreshold::get()
            };

            proposal.executed = true;
            Proposals::<T>::insert(proposal_id, proposal);

            // 从活跃提案中移除
            ActiveProposals::<T>::mutate(|proposals| {
                proposals.retain(|&id| id != proposal_id);
            });

            // 触发事件
            Self::deposit_event(Event::ProposalExecuted {
                proposal_id,
                success,
            });

            Ok(())
        }
    }

    // 内部函数
    impl<T: Config> Pallet<T> {
        /// 关闭过期提案的内部函数
        fn close_proposal(proposal_id: ProposalId, current_block: T::BlockNumber) -> Weight {
            if let Some(proposal) = Self::proposals(proposal_id) {
                if proposal.voting_deadline <= current_block && !proposal.executed {
                    // 从活跃提案中移除
                    ActiveProposals::<T>::mutate(|proposals| {
                        proposals.retain(|&id| id != proposal_id);
                    });

                    Self::deposit_event(Event::ProposalClosed { proposal_id });

                    return T::DbWeight::get().writes(1);
                }
            }
            Weight::zero()
        }
    }
}
```

### 4. 将 Pallet 集成到 Runtime

更新 `runtime/src/lib.rs`：

```rust
// 在 use 语句中添加
pub use pallet_voting;

// 在 construct_runtime! 宏中添加
Template: pallet_template,
Voting: pallet_voting,

// 在 runtime::runtime enum 中添加
Voting: pallet_voting,

// 添加 Voting Pallet 配置
#[cfg_attr(feature = "std", derive(frame_support::derive_impl::Debug))]
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
#[frame_support::pallet]
pub mod pallet_voting {
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent = RuntimeEvent;
        type MaxTitleLength = ConstU32<100>;
        type MaxDescriptionLength = ConstU32<1000>;
        type MaxActiveProposals = ConstU32<100>;
        type VoteThreshold = ConstU32>60>; // 60% 通过阈值
        type MinVotingDuration = ConstU32<100>; // 最少 100 个区块
        type MaxVotingDuration = ConstU32<1000>; // 最多 1000 个区块
    }
}
```

## 测试和部署

### 1. 运行测试

```bash
# 运行所有测试
cargo test --all

# 运行特定 pallet 的测试
cargo test -p pallet-voting

# 运行 runtime 测试
cargo test -p node-template-runtime
```

### 2. 构建和启动节点

```bash
# 构建节点
cargo build --release

# 启动开发节点
./target/release/node-template --dev --tmp

# 指定数据目录
./target/release/node-template --dev --base-path /tmp/node-data
```

### 3. 使用 Polkadot.js 进行交互

1. 访问 https://polkadot.js.org/apps/
2. 连接到本地节点：`ws://127.0.0.1:9944`
3. 在 "Developer" -> "Extrinsics" 中调用 pallet 函数
4. 在 "Developer" -> "Chain State" 中查询状态

## 项目文档模板

### 1. README.md 结构

```markdown
# Substrate 投票系统

## 项目概述

这是一个基于 Substrate 框架开发的区块链投票系统，支持创建提案、投票和执行功能。

## 功能特性

- 账户管理和身份验证
- 提案创建和管理
- 安全的投票机制
- 自动化的提案执行
- 完整的交易处理

## 架构设计

### Runtime 架构
- 使用 FRAME 框架进行模块化开发
- 集成标准 pallets：System, Balances, Timestamp, Aura, Grandpa
- 自定义 Voting Pallet 实现投票逻辑

### Pallet 设计
- **Voting Pallet**：处理提案创建、投票和执行
- **System Pallet**：提供基础系统功能
- **Balances Pallet**：管理账户余额

## 安装和运行

### 环境要求
- Rust 1.70+
- 16GB+ RAM
- Ubuntu 20.04+ 或 macOS

### 安装步骤
1. 安装 Rust 和 Substrate 工具链
2. 克隆项目并构建
3. 启动本地节点

### 使用指南
- 使用 Polkadot.js 进行交互
- 通过 RPC API 进行集成
- 编写自定义前端应用

## 开发指南

### 添加新功能
1. 创建新的 Pallet
2. 在 Runtime 中集成
3. 编写测试用例
4. 更新文档

### 部署流程
1. 测试环境验证
2. 代码审查
3. 安全审计
4. 生产部署

## API 文档

### RPC Endpoints
- `voting_createProposal`: 创建提案
- `voting_vote`: 投票
- `voting_executeProposal`: 执行提案

### Storage Queries
- `voting_proposals`: 查询提案
- `voting_votes`: 查询投票记录
- `voting_activeProposals`: 查询活跃提案

## 安全考虑

- 权限控制和身份验证
- 防止重复投票机制
- 时间限制和截止日期
- 算术溢出保护

## 性能优化

- 存储优化策略
- 计算复杂度分析
- 网络优化建议

## 社区和支持

- 问题反馈：GitHub Issues
- 技术讨论：Discord/Element
- 文档更新：Wiki

## 许可证

MIT License
```

## 常见问题和解决方案

### 1. 编译错误
```bash
# 清理缓存
cargo clean

# 更新依赖
cargo update

# 检查 Rust 版本
rustup show
```

### 2. 运行时错误
- 检查存储键冲突
- 验证类型转换
- 确认 pallet 配置

### 3. 性能问题
- 优化存储结构
- 减少不必要的计算
- 使用缓存机制

## 下一步扩展

1. **治理功能**：添加更复杂的治理机制
2. **身份系统**：集成去中心化身份
3. **跨链互操作**：支持与其他链的交互
4. **隐私保护**：添加隐私保护功能
5. **代币经济**：设计激励机制