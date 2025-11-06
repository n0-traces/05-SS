# 多链合约开发作业详细指南

## 概览

本作业要求学生掌握 Solana 智能合约开发和 Substrate 区块链框架开发，涵盖理论知识和实践操作两个方面。建议总时长：20-30 小时。

## 项目结构建议

```
task04/
├── README.md                    # 总体介绍和导航
├── docs/                        # 文档目录
│   ├── solana-theory.md         # Solana 理论题答案
│   ├── substrate-theory.md      # Substrate 理论题答案
│   └── deployment-guide.md      # 部署指南汇总
├── solana/                      # Solana 项目
│   ├── src/                     # Rust 源代码
│   ├── tests/                   # 测试文件
│   ├── Anchor.toml             # Anchor 配置
│   ├── Cargo.toml              # Rust 项目配置
│   └── README.md               # Solana 项目说明
├── substrate/                   # Substrate 项目
│   ├── runtime/                # Runtime 源代码
│   ├── pallets/                # 自定义 Pallets
│   ├── node/                   # 节点代码
│   ├── Cargo.toml              # 项目配置
│   └── README.md               # Substrate 项目说明
└── scripts/                     # 部署和测试脚本
    ├── deploy-solana.sh        # Solana 部署脚本
    └── run-substrate.sh        # Substrate 运行脚本
```

## 第一部分：Solana 智能合约开发

### 理论知识指导

#### 1. Sealevel 运行时分析要点

**研究角度：**
- 并行执行机制 vs 传统 EVM 串行执行
- 状态独立性如何实现并行处理
- 账户模型对并行执行的支持
- 实际应用场景：DeFi 协议、NFT 市场、游戏等

**答题框架：**
```
1. Sealevel 核心特性介绍
2. 与传统执行模型的对比分析
3. 具体应用场景案例（至少2个）
4. 性能优势的具体数据和指标
5. 开发者体验的影响
```

#### 2. BPF 虚拟机分析要点

**研究角度：**
- BPF（Berkeley Packet Filter）在 Solana 中的应用
- 安全性：沙箱机制、内存管理、执行限制
- 可扩展性：性能优化、资源控制
- 实际案例：Solana 程序的安全漏洞分析

**案例建议：**
- 案例一：内存安全保护机制
- 案例二：计算资源限制与优化

### 实践操作指导

#### 1. 开发环境搭建

**必需工具：**
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"

# 安装 Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# 安装 SPL CLI
cargo install spl-token-cli
```

#### 2. SPL 代币程序实现步骤

**步骤一：项目初始化**
```bash
anchor init spl-token-program
cd spl-token-program
```

**步骤二：核心功能实现**
- 创建代币铸造账户
- 实现代币铸造功能
- 实现代币转账功能
- 实现余额查询功能
- 添加权限控制

**步骤三：测试和部署**
- 编写单元测试
- 本地网测试
- Devnet 部署测试

### 关键代码结构建议

```rust
// lib.rs 主要结构
#[program]
pub mod spl_token_program {
    use super::*;

    pub fn create_mint(
        ctx: Context<CreateMint>,
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
    ) -> Result<()>

    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()>

    pub fn transfer_tokens(
        ctx: Context<TransferTokens>,
        amount: u64,
    ) -> Result<()>
}
```

## 第二部分：Substrate 开发

### 理论知识指导

#### 1. FRAME 模块分析要点

**研究角度：**
- FRAME (Framework for Runtime Aggregation of Modularized Entities) 设计理念
- 模块化开发的优势
- 与 balances 模块的深入分析
- 开发效率提升的具体体现

**答题框架：**
```
1. FRAME 架构设计原理
2. 模块化开发的优势分析
3. balances 模块深度解析
4. 开发效率对比（传统 vs FRAME）
5. 生态系统支持
```

#### 2. Pallet 设计模式分析要点

**研究角度：**
- Pallet 设计模式的核心概念
- 与传统区块链模块的差异
- 可组合性和互操作性
- 自定义功能实现的优势

### 实践操作指导

#### 1. 开发环境搭建

**必需工具：**
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Substrate 相关工具
curl https://getsubstrate.io -sSf | bash -s -- --fast

# 或使用 cargo 安装
cargo install --force substrate-contracts-node --version 0.31.0
cargo install --force cargo-contract --version 3.2.0
```

#### 2. 项目创建步骤

**步骤一：创建节点模板**
```bash
# 使用 substrate-node-template
git clone https://github.com/substrate-developer-hub/substrate-node-template
cd substrate-node-template
```

**步骤二：实现基础功能**
- 账户管理功能
- 交易处理逻辑
- 区块生成机制

**步骤三：添加自定义 Pallet**
- 创建投票 Pallet 或资产登记 Pallet
- 实现 Pallet 的核心逻辑
- 集成到 Runtime 中

### 自定义 Pallet 实现建议

**投票 Pallet 结构：**
```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    type VoteThreshold: Get<u32>;
}

#[pallet::storage]
#[pallet::getter(fn proposals)]
pub type Proposals<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    ProposalId,
    Proposal<T::AccountId, T::BlockNumber>,
    ValueQuery,
>;

#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    ProposalCreated(ProposalId, T::AccountId),
    Voted(ProposalId, T::AccountId, bool),
}
```

## 时间规划和里程碑

### 第1-2周：理论学习与准备
- [ ] 完成 Solana 理论研究
- [ ] 完成 Substrate 理论研究
- [ ] 搭建开发环境
- [ ] 完成理论题答案撰写

### 第3-4周：Solana 实践开发
- [ ] 创建 Solana 项目结构
- [ ] 实现 SPL 代币核心功能
- [ ] 编写测试用例
- [ ] 完成部署文档

### 第5-6周：Substrate 实践开发
- [ ] 创建 Substrate 节点项目
- [ ] 实现基础 Runtime 功能
- [ ] 开发自定义 Pallet
- [ ] 完成项目文档

### 第7周：整合与优化
- [ ] 整合所有项目文档
- [ ] 代码优化和注释完善
- [ ] 最终测试和验证
- [ ] 准备提交材料

## 评分标准参考

### 理论部分（40%）
- 概念理解准确性：15%
- 分析深度和逻辑性：15%
- 案例质量和相关性：10%

### 实践部分（60%）
- 代码质量和功能完整性：25%
- 文档详细程度和规范性：20%
- 创新性和扩展性：15%

## 资源推荐

### Solana 相关
- [Solana 官方文档](https://docs.solana.com/)
- [Anchor 框架文档](https://anchor-lang.com/)
- [SPL Token Program](https://spl.solana.com/token)

### Substrate 相关
- [Substrate 开发者中心](https://substrate.io/developers/)
- [Substrate 文档](https://docs.substrate.io/)
- [FRAME 支持 pallets](https://substrate.io/rustdocs/latest/frame_support/index.html)

### 工具和社区
- [Rust 学习资源](https://www.rust-lang.org/learn)
- Solana Discord 社区
- Substrate Element 聊天室

## 注意事项

1. **版本兼容性**：确保使用稳定版本的工具链
2. **安全性考虑**：在智能合约中特别注意安全漏洞
3. **文档质量**：代码注释和文档同样重要
4. **测试覆盖**：确保关键功能都有相应的测试
5. **时间管理**：合理分配时间，避免前松后紧

## 提交检查清单

- [ ] 理论题答案完整且准确
- [ ] Solana 项目可正常运行
- [ ] Substrate 节点可正常启动
- [ ] 所有代码都有详细注释
- [ ] 文档结构清晰、格式规范
- [ ] 部署和运行说明详细
- [ ] 项目结构合理、文件组织清晰
- [ ] 包含必要的测试用例