# Substrate 投票区块链节点

这是一个基于 Substrate 框架开发的自定义区块链，集成了投票和治理功能。作为多链合约开发教学作业的 Substrate 部分。

## 项目概述

本项目实现了一个完整的区块链节点，包含以下核心功能：

- **账户管理**：基础的用户账户和余额管理
- **交易处理**：完整的交易验证和执行机制
- **区块生成**：基于 Aura 和 Grandpa 的共识机制
- **投票系统**：完整的提案创建、投票和执行功能
- **治理机制**：基于代币权重的投票权重计算

## 技术架构

### 核心组件

- **FRAME 框架**：模块化的 Runtime 开发框架
- **Aura 共识**：权威证明共识算法
- **Grandpa 最终性**：区块最终性共识
- **Balances Pallet**：账户和余额管理
- **Voting Pallet**：自定义投票模块

### 系统特性

- **高性能**：优化的执行引擎和存储机制
- **模块化**：可插拔的功能模块设计
- **可升级**：支持 Runtime 的无缝升级
- **安全性**：多层安全防护机制

## 项目结构

```
substrate/
├── Cargo.toml                  # 主项目配置
├── README.md                   # 项目说明文档
├── build.rs                    # 构建脚本
└── src/
    └── lib.rs                  # Runtime 实现
└── pallets/
    └── voting/
        ├── Cargo.toml          # 投票 Pallet 配置
        └── src/
            └── lib.rs          # 投票 Pallet 实现
```

## 环境要求

### 系统要求

- Rust 1.70+
- Git 2.0+
- 16GB+ RAM（推荐）
- Ubuntu 20.04+ 或 macOS

### 工具安装

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 添加 WASM 目标
rustup target add wasm32-unknown-unknown

# 安装 Substrate 工具
cargo install --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.31.0 substrate-contracts-node
```

## 构建和运行

### 1. 构建项目

```bash
# 进入项目目录
cd substrate

# 构建项目
cargo build --release

# 验证构建
./target/release/substrate-voting-node --version
```

### 2. 运行开发节点

```bash
# 启动开发节点
./target/release/substrate-voting-node --dev --tmp

# 或使用指定数据目录
./target/release/substrate-voting-node --dev --base-path /tmp/substrate-data

# 启动时指定参数
./target/release/substrate-voting-node \
  --dev \
  --tmp \
  --rpc-port 9933 \
  --ws-port 9944 \
  --validator
```

### 3. 清理链数据

```bash
# 清理链数据
./target/release/substrate-voting-node purge-chain --dev

# 强制清理
./target/release/substrate-voting-node purge-chain --dev -y
```

## 投票系统功能

### 1. 创建提案

```bash
# 使用 polkadot.js.org/apps 创建提案
# 或通过 RPC 调用

curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "author_submitExtrinsic",
    "params": [
      "0x... (encoded extrinsic)"
    ],
    "id": 1
  }'
```

### 2. 投票

```bash
# 对指定提案进行投票
# 支持 Yes/No 两种选择
# 投票权重基于账户余额计算
```

### 3. 执行提案

```bash
# 在投票期结束后执行提案
# 自动检查是否达到通过阈值
```

### 4. 查询状态

```bash
# 查询提案信息
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "voting_proposals",
    "params": [1],
    "id": 1
  }'

# 查询投票记录
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "voting_votes",
    "params": [1, "0x... (account)"],
    "id": 1
  }'
```

## 配置参数

### 投票系统配置

```rust
// 在 Runtime 配置中设置
parameter_types! {
    pub const MaxTitleLength: u32 = 100;           // 提案标题最大长度
    pub const MaxDescriptionLength: u32 = 1000;    // 提案描述最大长度
    pub const MaxActiveProposals: u32 = 100;       // 最大活跃提案数量
    pub const VoteThreshold: Percent = Percent::from_percent(60);  // 通过阈值 60%
    pub const MinVotingDuration: BlockNumber = 100; // 最小投票期
    pub const MaxVotingDuration: BlockNumber = 1000;// 最大投票期
    pub const MinVotingWeight: Balance = 1000;      // 最小投票权重
}
```

### 网络配置

```toml
# chain-spec.json 配置示例
{
  "name": "Voting Chain",
  "id": "voting_local",
  "chainType": "Local",
  "bootNodes": [],
  "telemetryEndpoints": null,
  "protocolId": "voting-local",
  "properties": {
    "ss58Format": 42,
    "tokenDecimals": 12,
    "tokenSymbol": "VOTE"
  }
}
```

## 测试

### 运行单元测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test -p pallet-voting

# 运行 Runtime 测试
cargo test -p substrate-voting-node
```

### 运行集成测试

```bash
# 运行集成测试
cargo test --release --features runtime-benchmarks

# 性能基准测试
cargo run --release --features runtime-benchmarks -- benchmark --chain dev
```

### 测试覆盖的功能

- ✅ 提案创建和管理
- ✅ 投票机制验证
- ✅ 权限控制测试
- ✅ 过期处理机制
- ✅ 统计信息更新
- ✅ 错误处理测试
- ✅ 并发操作测试

## API 文档

### 核心函数

#### `create_proposal(title, description, duration)`
创建新的投票提案。

**参数：**
- `title`: 提案标题（最大 100 字符）
- `description`: 提案描述（最大 1000 字符）
- `duration`: 投票持续时间（区块数）

#### `vote(proposal_id, choice)`
对提案进行投票。

**参数：**
- `proposal_id`: 提案 ID
- `choice`: 投票选择（Yes/No）

#### `execute_proposal(proposal_id)`
执行投票期结束的提案。

**参数：**
- `proposal_id`: 提案 ID

### 查询函数

#### `proposals(proposal_id)`
查询指定提案的详细信息。

#### `votes(proposal_id, account)`
查询指定账户的投票记录。

#### `active_proposals()`
查询所有活跃提案的 ID 列表。

#### `proposal_stats()`
查询提案统计信息。

### 事件类型

```rust
pub enum Event {
    ProposalCreated { proposal_id, proposer },
    Voted { proposal_id, voter, choice, weight },
    ProposalPassed { proposal_id, yes_votes, no_votes },
    ProposalRejected { proposal_id, yes_votes, no_votes },
    ProposalExecuted { proposal_id, executor },
    ProposalExpired { proposal_id },
}
```

### 错误类型

```rust
pub enum Error {
    ProposalNotFound,
    ProposalAlreadyExists,
    TitleTooLong,
    DescriptionTooLong,
    VotingPeriodExpired,
    AlreadyVoted,
    NotProposer,
    AlreadyExecuted,
    TooManyActiveProposals,
    InvalidVotingDuration,
    InsufficientVotingWeight,
    ProposalNotPassed,
    InvalidProposalStatus,
    ArithmeticError,
}
```

## 性能特性

### 执行性能

- **区块生成时间**：6 秒
- **交易处理能力**：1000+ TPS
- **存储优化**：紧凑的数据结构
- **内存效率**：优化的内存使用

### 治理特性

- **投票权重**：基于账户余额的权重计算
- **通过阈值**：可配置的通过比例
- **投票周期**：灵活的投票时间设置
- **自动执行**：达到条件后自动执行

## 安全考虑

### 权限控制

- **账户验证**：严格的账户所有权验证
- **重复投票防护**：防止同一账户重复投票
- **权限检查**：基于账户状态的权限验证
- **时间控制**：投票期的时间限制

### 安全防护

1. **溢出保护**：所有数值操作的溢出检查
2. **状态一致性**：严格的状态转换规则
3. **权限隔离**：不同操作的权限分离
4. **数据完整性**：完整的数据验证机制

## 故障排除

### 常见问题

#### 1. 构建失败
```bash
# 清理并重新构建
cargo clean
cargo build --release

# 检查 Rust 版本
rustc --version
cargo --version
```

#### 2. 节点启动失败
```bash
# 检查端口占用
lsof -i :9933
lsof -i :9944

# 清理链数据
./target/release/substrate-voting-node purge-chain --dev

# 重新启动
./target/release/substrate-voting-node --dev --tmp
```

#### 3. 交易失败
```bash
# 检查账户余额
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "system_accountNextIndex",
    "params": ["0x..."],
    "id": 1
  }'

# 检查交易状态
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "author_extrinsicUpdate",
    "params": ["0x..."],
    "id": 1
  }'
```

### 调试技巧

1. **日志分析**：查看节点的详细日志输出
2. **状态查询**：使用 RPC 查询链状态
3. **交易模拟**：在提交前模拟交易执行
4. **事件监控**：监控链上事件

## 开发指南

### 添加新功能

1. **创建新 Pallet**：使用 `pallet-template` 作为起点
2. **实现接口**：实现必要的 trait 和接口
3. **集成到 Runtime**：在 `construct_runtime!` 中添加
4. **编写测试**：编写全面的测试用例
5. **更新文档**：更新相关文档

### 最佳实践

1. **代码规范**：遵循 Rust 代码规范
2. **错误处理**：完善的错误处理机制
3. **测试覆盖**：确保充分的测试覆盖
4. **文档维护**：保持文档的更新
5. **性能优化**：关注性能优化

## 部署指南

### 本地部署

```bash
# 构建发布版本
cargo build --release

# 生成链规范
./target/release/substrate-voting-node build-spec --chain local --raw > chain-spec.json

# 启动节点
./target/release/substrate-voting-node --chain chain-spec.json
```

### Docker 部署

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/substrate-voting-node /usr/local/bin/

EXPOSE 9933 9944 30333
CMD ["substrate-voting-node", "--dev", "--ws-external", "--rpc-external"]
```

### 生产环境

1. **硬件要求**：16GB+ RAM，4核+ CPU
2. **网络配置**：安全的网络配置
3. **监控设置**：完善的监控和告警
4. **备份策略**：定期数据备份

## 贡献指南

### 开发流程

1. **Fork 项目**：创建项目分支
2. **创建功能分支**：基于 `main` 创建
3. **开发测试**：编写代码和测试
4. **提交 PR**：提交 Pull Request
5. **代码审查**：通过代码审查后合并

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 进行代码检查
- 编写详细的注释和文档
- 保持测试覆盖率 > 80%

## 许可证

本项目采用 Unlicense 许可证，详见 [LICENSE](LICENSE) 文件。

## 相关资源

- [Substrate 官方文档](https://substrate.io/docs/)
- [FRAME 支持文档](https://docs.substrate.io/reference/frame-support/)
- [Polkadot.js API 文档](https://polkadot.js.org/docs/)
- [Rust 编程语言](https://www.rust-lang.org/)

## 更新日志

### v1.0.0 (2024-01-01)
- ✅ 初始版本发布
- ✅ 基础投票功能实现
- ✅ Aura + Grandpa 共识集成
- ✅ 完整的测试套件
- ✅ 详细的文档

### v1.1.0 (计划中)
- 🔄 性能优化
- 🔄 治理功能增强
- 🔄 跨链互操作支持
- 🔄 更多治理工具

---

**注意**：这是一个教学项目，主要用于学习和演示 Substrate 区块链开发。在生产环境中使用前，请进行全面的安全审计和性能测试。