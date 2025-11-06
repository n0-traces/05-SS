//! # Voting Pallet
//!
//! 这个 pallet 提供了一个基础的投票系统，允许用户创建提案并进行投票。
//!
//! ## 功能特性
//!
//! - **提案创建**：用户可以创建投票提案
//! - **投票机制**：支持赞成/反对投票
//! - **自动执行**：达到阈值后自动执行提案
//! - **权限控制**：防止重复投票和非法操作
//! - **过期处理**：自动处理过期提案

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement, Get},
    BoundedVec,
};
use frame_system::ensure_signed;
use sp_runtime::{ArithmeticError, DispatchError, Percent};
use sp_std::{vec::Vec, prelude::*};

pub use pallet::*;

/// 提案 ID 类型
pub type ProposalId = u32;

/// 投票选择类型
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum VoteChoice {
    /// 赞成
    Yes,
    /// 反对
    No,
}

/// 提案状态
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum ProposalStatus {
    /// 活跃状态（可以投票）
    Active,
    /// 已通过
    Passed,
    /// 已拒绝
    Rejected,
    /// 已执行
    Executed,
    /// 已过期
    Expired,
}

/// 提案结构体
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
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
    /// 赞成票数
    pub yes_votes: u32,
    /// 反对票数
    pub no_votes: u32,
    /// 总投票权重
    pub total_weight: u128,
    /// 提案状态
    pub status: ProposalStatus,
    /// 是否已执行
    pub executed: bool,
    /// 执行时间
    pub executed_at: Option<BlockNumber>,
}

/// 投票记录
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Vote<AccountId, Balance> {
    /// 投票者
    pub voter: AccountId,
    /// 投票选择
    pub choice: VoteChoice,
    /// 投票权重（基于账户余额）
    pub weight: Balance,
    /// 投票时间
    pub voted_at: frame_system::pallet_prelude::BlockNumberFor<T>,
}

// 类型别名
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Runtime event 类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型，用于计算投票权重
        type Currency: Currency<Self::AccountId>;

        /// 提案标题最大长度
        #[pallet::constant]
        type MaxTitleLength: Get<u32>;

        /// 提案描述最大长度
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// 最大活跃提案数量
        #[pallet::constant]
        type MaxActiveProposals: Get<u32>;

        /// 投票通过阈值（百分比）
        #[pallet::constant]
        type VoteThreshold: Get<Percent>;

        /// 最小投票持续时间
        #[pallet::constant]
        type MinVotingDuration: Get<Self::BlockNumber>;

        /// 最大投票持续时间
        #[pallet::constant]
        type MaxVotingDuration: Get<Self::BlockNumber>;

        /// 最小投票权重要求
        #[pallet::constant]
        type MinVotingWeight: Get<BalanceOf<Self>>;

        /// 权重信息
        type WeightInfo: frame_support::traits::WeightInfo;
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
        Blake2_128Concat,
        ProposalId,
        Proposal<AccountIdOf<T>, BlockNumberOf<T>>,
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
        Blake2_128Concat,
        ProposalId,
        Blake2_128Concat,
        AccountIdOf<T>,
        Vote<AccountIdOf<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    /// 存储每个账户的投票记录（防止重复投票）
    #[pallet::storage]
    #[pallet::getter(fn voter_proposals)]
    pub type VoterProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        AccountIdOf<T>,
        BoundedVec<ProposalId, ConstU32<100>>,
        ValueQuery,
    >;

    /// 存储提案统计信息
    #[pallet::storage]
    #[pallet::getter(fn proposal_stats)]
    pub type ProposalStats<T: Config> = StorageValue<
        _,
        ProposalStatsInfo<BlockNumberOf<T>>,
        ValueQuery,
    >;

    /// 提案统计信息结构
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Default)]
    pub struct ProposalStatsInfo<BlockNumber> {
        /// 总提案数
        pub total_proposals: ProposalId,
        /// 通过的提案数
        pub passed_proposals: u32,
        /// 拒绝的提案数
        pub rejected_proposals: u32,
        /// 执行的提案数
        pub executed_proposals: u32,
        /// 最后更新时间
        pub last_updated: BlockNumber,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 新提案创建
        ProposalCreated {
            proposal_id: ProposalId,
            proposer: AccountIdOf<T>,
        },
        /// 投票完成
        Voted {
            proposal_id: ProposalId,
            voter: AccountIdOf<T>,
            choice: VoteChoice,
            weight: BalanceOf<T>,
        },
        /// 提案通过
        ProposalPassed {
            proposal_id: ProposalId,
            yes_votes: u32,
            no_votes: u32,
        },
        /// 提案拒绝
        ProposalRejected {
            proposal_id: ProposalId,
            yes_votes: u32,
            no_votes: u32,
        },
        /// 提案执行
        ProposalExecuted {
            proposal_id: ProposalId,
            executor: AccountIdOf<T>,
        },
        /// 提案过期
        ProposalExpired {
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
        /// 投票权重不足
        InsufficientVotingWeight,
        /// 提案未通过
        ProposalNotPassed,
        /// 提案状态不允许此操作
        InvalidProposalStatus,
        /// 算术错误
        ArithmeticError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberOf<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberOf<T>) -> Weight {
            // 检查并关闭过期的提案
            let expired_proposals = Self::active_proposals()
                .into_iter()
                .filter(|&id| {
                    if let Some(proposal) = Self::proposals(id) {
                        proposal.voting_deadline <= n && proposal.status == ProposalStatus::Active
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();

            let mut weight_used = Weight::zero();
            for proposal_id in expired_proposals {
                weight_used += Self::expire_proposal(proposal_id, n);
            }

            weight_used
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新提案
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_proposal())]
        pub fn create_proposal(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Vec<u8>,
            voting_duration: BlockNumberOf<T>,
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
                total_weight: 0,
                status: ProposalStatus::Active,
                executed: false,
                executed_at: None,
            };

            // 存储提案
            Proposals::<T>::insert(proposal_id, proposal);

            // 添加到活跃提案列表
            ActiveProposals::<T>::try_mutate(|proposals| {
                proposals.try_push(proposal_id)
            }).map_err(|_| Error::<T>::TooManyActiveProposals)?;

            // 更新统计信息
            Self::update_stats(|stats| {
                stats.total_proposals = new_id;
                stats.last_updated = current_block;
            });

            // 触发事件
            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                proposer,
            });

            Ok(())
        }

        /// 对提案投票
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::vote())]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
            choice: VoteChoice,
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

            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::InvalidProposalStatus
            );

            // 检查是否已经投过票
            ensure!(
                !Votes::<T>::contains_key(proposal_id, &voter),
                Error::<T>::AlreadyVoted
            );

            // 计算投票权重（基于账户余额）
            let balance = T::Currency::free_balance(&voter);
            ensure!(
                balance >= T::MinVotingWeight::get(),
                Error::<T>::InsufficientVotingWeight
            );

            // 创建投票记录
            let vote = Vote {
                voter: voter.clone(),
                choice,
                weight: balance,
                voted_at: current_block,
            };

            // 存储投票
            Votes::<T>::insert(proposal_id, &voter, vote);

            // 更新提案的投票计数
            match choice {
                VoteChoice::Yes => {
                    proposal.yes_votes = proposal.yes_votes.checked_add(1)
                        .ok_or(ArithmeticError::Overflow)?;
                }
                VoteChoice::No => {
                    proposal.no_votes = proposal.no_votes.checked_add(1)
                        .ok_or(ArithmeticError::Overflow)?;
                }
            }

            proposal.total_weight = proposal.total_weight.checked_add(balance as u128)
                .ok_or(ArithmeticError::Overflow)?;

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
                weight: balance,
            });

            Ok(())
        }

        /// 执行提案
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::execute_proposal())]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
        ) -> DispatchResult {
            let executor = ensure_signed(origin)?;

            let mut proposal = Self::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(!proposal.executed, Error::<T>::AlreadyExecuted);

            let current_block = frame_system::Pallet::<T>::block_number();

            // 检查投票是否结束
            if current_block <= proposal.voting_deadline {
                return Err(Error::<T>::VotingPeriodExpired.into());
            }

            // 检查提案是否通过
            let threshold = T::VoteThreshold::get();
            let total_votes = proposal.yes_votes + proposal.no_votes;

            let passed = if total_votes == 0 {
                false
            } else {
                let yes_percentage = Percent::from_rational(proposal.yes_votes, total_votes);
                yes_percentage >= threshold
            };

            // 更新提案状态
            proposal.status = if passed {
                ProposalStatus::Passed
            } else {
                ProposalStatus::Rejected
            };
            proposal.executed = true;
            proposal.executed_at = Some(current_block);

            Proposals::<T>::insert(proposal_id, proposal);

            // 从活跃提案中移除
            ActiveProposals::<T>::mutate(|proposals| {
                proposals.retain(|&id| id != proposal_id);
            });

            // 更新统计信息
            Self::update_stats(|stats| {
                if passed {
                    stats.passed_proposals += 1;
                } else {
                    stats.rejected_proposals += 1;
                }
                stats.executed_proposals += 1;
                stats.last_updated = current_block;
            });

            // 触发相应事件
            if passed {
                Self::deposit_event(Event::ProposalPassed {
                    proposal_id,
                    yes_votes: proposal.yes_votes,
                    no_votes: proposal.no_votes,
                });
            } else {
                Self::deposit_event(Event::ProposalRejected {
                    proposal_id,
                    yes_votes: proposal.yes_votes,
                    no_votes: proposal.no_votes,
                });
            }

            Self::deposit_event(Event::ProposalExecuted {
                proposal_id,
                executor,
            });

            Ok(())
        }
    }

    // 内部函数
    impl<T: Config> Pallet<T> {
        /// 关闭过期提案的内部函数
        fn expire_proposal(proposal_id: ProposalId, current_block: BlockNumberOf<T>) -> Weight {
            if let Some(mut proposal) = Self::proposals(proposal_id) {
                if proposal.voting_deadline <= current_block && proposal.status == ProposalStatus::Active {
                    // 更新提案状态
                    proposal.status = ProposalStatus::Expired;
                    Proposals::<T>::insert(proposal_id, proposal);

                    // 从活跃提案中移除
                    ActiveProposals::<T>::mutate(|proposals| {
                        proposals.retain(|&id| id != proposal_id);
                    });

                    // 更新统计信息
                    Self::update_stats(|stats| {
                        stats.last_updated = current_block;
                    });

                    Self::deposit_event(Event::ProposalExpired { proposal_id });

                    return T::WeightInfo::expire_proposal();
                }
            }
            Weight::zero()
        }

        /// 更新提案统计信息
        fn update_stats<F>(updater: F)
        where
            F: FnOnce(&mut ProposalStatsInfo<BlockNumberOf<T>>),
        {
            ProposalStats::<T>::mutate(|stats| {
                updater(stats);
            });
        }

        /// 检查提案是否通过
        pub fn is_proposal_passed(proposal_id: ProposalId) -> bool {
            if let Some(proposal) = Self::proposals(proposal_id) {
                if proposal.status == ProposalStatus::Active {
                    return false;
                }

                let threshold = T::VoteThreshold().get();
                let total_votes = proposal.yes_votes + proposal.no_votes;

                if total_votes == 0 {
                    return false;
                }

                let yes_percentage = Percent::from_rational(proposal.yes_votes, total_votes);
                yes_percentage >= threshold
            } else {
                false
            }
        }

        /// 获取用户的投票记录
        pub fn get_user_vote(proposal_id: ProposalId, user: &AccountIdOf<T>) -> Option<Vote<AccountIdOf<T>, BalanceOf<T>>> {
            Self::votes(proposal_id, user)
        }

        /// 获取提案的投票统计
        pub fn get_proposal_vote_stats(proposal_id: ProposalId) -> Option<(u32, u32, u128)> {
            Self::proposals(proposal_id).map(|proposal| {
                (proposal.yes_votes, proposal.no_votes, proposal.total_weight)
            })
        }
    }
}