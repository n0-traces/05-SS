//! Substrate Voting Node Runtime
//!
//! 这是基于 Substrate 框架的自定义区块链 Runtime，支持投票和治理功能。

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` 宏执行大量代码生成。
#![recursion_limit = "256"]

// 增加编译器对 pallet 生成的代码的可见性。
pub use frame_support::traits::{
    ConstU128, ConstU32, ConstU64, ConstU8, Currency, ExistenceRequirement, Get,
    Imbalance, OnUnbalanced, Randomness,
};
pub use frame_support::{
    construct_runtime, dispatch::DispatchResult, parameter_types,
    traits::FindAuthor, weights::constants::BlockExecutionWeight,
    weights::constants::ExtrinsicBaseWeight, weights::ConstantMultiplier,
    weights::IdentityFee, weights::Weight, StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_sudo::Call as SudoCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_voting::{Call as VotingCall, VoteChoice, ProposalStatus};

use codec::{Decode, Encode, MaxEncodedLen};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_core::H256;
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, DispatchError, KeyTypeId, MultiSignature, MultiSigner, OpaqueExtrinsic,
    Perbill, Permill, SaturatedConversion,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

/// 类型别名
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Hash = H256;
pub type Signature = MultiSignature;

/// Opaque block header type
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Opaque block type
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Opaque block identifier type
pub type BlockId = generic::BlockId<Block>;
/// Opaque Signed transaction type
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

/// SignedExtra 是签名者必须为 extrinsic 提供的额外信息。
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// UncheckedExtrinsic 是 extrinsic 的未检查版本。
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic 是交易/消息的已检查版本。
pub type Extrinsic = generic::Extrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Executive 处理区块
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// 运行时版本。
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("substrate-voting-node"),
    impl_name: create_runtime_str!("substrate-voting-node"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// 该运行时版本的 Native 版本。
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    /// 基础调用过滤器
    type BaseCallFilter = frame_support::traits::Everything;
    /// BlockWeights 配置
    type BlockWeights = frame_system::weights::constants::DefaultBlockWeights;
    /// BlockLength 配置
    type BlockLength = frame_system::weights::constants::DefaultBlockLength;
    /// AccountId
    type AccountId = AccountId;
    /// 用于系统 extrinsic 的调用
    type RuntimeCall = RuntimeCall;
    /// 用于查找的 Lookup
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// Index 类型
    type Index = u32;
    /// Block number 类型
    type BlockNumber = BlockNumber;
    /// Hash 类型
    type Hash = Hash;
    /// Hashing 算法
    type Hashing = BlakeTwo256;
    /// Header 类型
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Runtime event
    type RuntimeEvent = RuntimeEvent;
    /// RuntimeOrigin
    type RuntimeOrigin = RuntimeOrigin;
    /// BlockHashCount
    type BlockHashCount = BlockHashCount;
    /// DbWeight
    type DbWeight = ();
    /// Version
    type Version = Version;
    /// PalletInfo
    type PalletInfo = PalletInfo;
    /// AccountData
    type AccountData = pallet_balances::AccountData<Balance>;
    /// OnNewAccount
    type OnNewAccount = ();
    /// OnKilledAccount
    type OnKilledAccount = ();
    /// SystemWeightInfo
    type SystemWeightInfo = ();
    /// SS58Prefix
    type SS58Prefix = SS58Prefix;
    /// OnSetCode
    type OnSetCode = ();
    /// MaxConsumers
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
    /// Moment 类型
    type Moment = u64;
    /// OnTimestampSet
    type OnTimestampSet = Aura;
    /// MinimumPeriod
    type MinimumPeriod = MinimumPeriod;
    /// WeightInfo
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxAuthorities: u32 = 32;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
    type MaxSetIdSessionEntries = ConstU32<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    /// Balance 类型
    type Balance = Balance;
    /// DustRemoval
    type DustRemoval = ();
    /// RuntimeEvent
    type RuntimeEvent = RuntimeEvent;
    /// ExistentialDeposit
    type ExistentialDeposit = ExistentialDeposit;
    /// AccountStore
    type AccountStore = System;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = ();
    type MaxHolds = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1;
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = ();
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
}

parameter_types! {
    pub const MaxTitleLength: u32 = 100;
    pub const MaxDescriptionLength: u32 = 1000;
    pub const MaxActiveProposals: u32 = 100;
    pub const VoteThreshold: Percent = Percent::from_percent(60);
    pub const MinVotingDuration: BlockNumber = 100;
    pub const MaxVotingDuration: BlockNumber = 1000;
    pub const MinVotingWeight: Balance = 1000;
}

impl pallet_voting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxTitleLength = MaxTitleLength;
    type MaxDescriptionLength = MaxDescriptionLength;
    type MaxActiveProposals = MaxActiveProposals;
    type VoteThreshold = VoteThreshold;
    type MinVotingDuration = MinVotingDuration;
    type MaxVotingDuration = MaxVotingDuration;
    type MinVotingWeight = MinVotingWeight;
    type WeightInfo = pallet_voting::WeightInfo;
}

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
        Voting: pallet_voting,
    }
);

// 运行时 API 定义
#[cfg(feature = "std")]
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

#[cfg(feature = "std")]
pub type BlockNumber = sp_runtime::traits::BlockNumberProvider;

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

/// Block identifier type.
pub type BlockNumber = sp_runtime::traits::BlockNumberProvider;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification. Note that `Block` does not have
/// `Justification` field by default and that's why this struct is needed.
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

#[cfg(feature = "std")]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    ApplyExtrinsic(u32),
    Finalization,
}

#[cfg(feature = "std")]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionPhaseResult {
    pub phase: Phase,
    pub blockWeight: Weight,
    pub extrinsicWeights: Vec<(u32, Weight)>,
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, u32> for Runtime {
        fn account_nonce(account: AccountId) -> u32 {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::Weight>,
        ) {
            use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
            use frame_support::traits::Weight;
            use frame_system_benchmarking::Pallet as SystemBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_voting, Voting);

            let storage_info = AllPalletsWithSystem::storage_weights();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b8ab6c48bab1a5c113144e14ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8593ea34e6b893c37f86b9bd66de9986773b5826398e259db4").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516929d93bdf8ce690c").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            use pallet_balances::benchmarking::Pallet as BalancesBench;
            use pallet_voting::benchmarking::Pallet as VotingBench;

            batches.push(BenchmarkBatch::new(
                "balances",
                BalancesBench::benchmark(*params).map_err(|e| e.into())?,
            ));

            batches.push(BenchmarkBatch::new(
                "voting",
                VotingBench::benchmark(*params).map_err(|e| e.into())?,
            ));

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}
