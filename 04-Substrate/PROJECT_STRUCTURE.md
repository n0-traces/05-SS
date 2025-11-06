# 项目结构和文件组织指南

## 完整项目目录结构

```
task04/
├── README.md                           # 项目总体介绍和导航
├── GUIDE.md                           # 详细任务指南
├── PROJECT_STRUCTURE.md               # 项目结构说明（本文件）
├── .gitignore                         # Git 忽略文件配置
├── requirements.txt                   # Python 依赖（如果需要）
└── package.json                       # Node.js 依赖（如果需要）

# 文档目录
├── docs/                              # 所有文档文件
│   ├── README.md                      # 文档索引
│   ├── solana-theory.md              # Solana 理论题答案
│   ├── substrate-theory.md           # Substrate 理论题答案
│   ├── solana-implementation-guide.md # Solana 实现指南
│   ├── substrate-implementation-guide.md # Substrate 实现指南
│   ├── deployment-guide.md           # 综合部署指南
│   ├── api-reference.md              # API 参考文档
│   ├── security-considerations.md    # 安全考虑
│   └── troubleshooting.md            # 故障排除指南

# Solana 项目目录
├── solana/                            # Solana SPL 代币项目
│   ├── README.md                      # Solana 项目说明
│   ├── .anchor/                       # Anchor 配置目录
│   ├── app/                          # 前端应用（可选）
│   ├── migrations/                   # 部署脚本
│   ├── programs/                     # 智能合约程序
│   │   └── solana-spl-token/
│   │       ├── Cargo.toml           # Rust 项目配置
│   │       ├── Xargo.toml           # 交叉编译配置
│   │       └── src/
│   │           ├── lib.rs           # 主要逻辑
│   │           ├── instruction.rs   # 指令定义
│   │           ├── processor.rs     # 处理逻辑
│   │           ├── state.rs         # 状态管理
│   │           └── error.rs         # 错误定义
│   ├── tests/                        # 测试文件
│   │   ├── solana-spl-token.ts      # TypeScript 测试
│   │   ├── utils.ts                 # 测试工具函数
│   │   └── fixtures/                # 测试数据
│   ├── scripts/                      # 构建和部署脚本
│   │   ├── build.sh                 # 构建脚本
│   │   ├── deploy.sh                # 部署脚本
│   │   └── test.sh                  # 测试脚本
│   ├── Anchor.toml                  # Anchor 项目配置
│   ├── Cargo.toml                   # 工作空间配置
│   ├── package.json                 # Node.js 配置
│   ├── tsconfig.json               # TypeScript 配置
│   └── yarn.lock                   # 依赖锁定文件

# Substrate 项目目录
├── substrate/                        # Substrate 区块链项目
│   ├── README.md                     # Substrate 项目说明
│   ├── node/                        # 节点实现
│   │   └── src/
│   │       ├── chain_spec.rs        # 链规范配置
│   │       ├── command.rs           # CLI 命令处理
│   │       ├── lib.rs               # 节点库入口
│   │       ├── main.rs              # 节点二进制入口
│   │       ├── rpc.rs               # RPC API 配置
│   │       ├── service.rs           # 节点服务实现
│   │       └── cli/                 # CLI 命令定义
│   ├── runtime/                     # Runtime 实现
│   │   ├── build.rs                 # 构建脚本
│   │   ├── Cargo.toml               # Runtime 配置
│   │   └── src/
│   │       ├── lib.rs               # Runtime 库入口
│   │       ├── runtime.rs           # Runtime 配置
│   │       ├── pallets/             # 自定义 Pallets
│   │       │   ├── mod.rs           # Pallets 模块
│   │       │   ├── template.rs      # 模板 Pallet
│   │       │   └── voting.rs        # 投票 Pallet
│   │       └── primitives.rs        # 基础类型定义
│   ├── pallets/                     # 自定义 Pallets 开发目录
│   │   ├── voting/                  # 投票 Pallet
│   │   │   ├── Cargo.toml           # Pallet 配置
│   │   │   └── src/
│   │   │       ├── lib.rs           # Pallet 实现
│   │   │       ├── mock.rs          # 测试 Mock
│   │   │       ├── tests.rs         # 单元测试
│   │   │       └── benchmarking.rs  # 性能测试
│   │   └── template/                # 模板 Pallet
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── lib.rs
│   │           ├── mock.rs
│   │           └── tests.rs
│   ├── scripts/                     # 构建和部署脚本
│   │   ├── build.sh                # 构建脚本
│   │   ├── run.sh                  # 运行脚本
│   │   ├── test.sh                 # 测试脚本
│   │   └── purge-chain.sh          # 清理数据脚本
│   ├── test/                       # 集成测试
│   │   └── integration/
│   ├── res/                        # 资源文件
│   ├── Cargo.toml                  # 工作空间配置
│   ├── rust-toolchain.toml         # Rust 工具链配置
│   └── .gitignore                  # Git 忽略文件

# 通用脚本和工具目录
├── scripts/                         # 项目级脚本
│   ├── setup.sh                    # 环境设置脚本
│   ├── build-all.sh               # 构建所有项目
│   ├── test-all.sh                # 测试所有项目
│   ├── deploy-all.sh              # 部署所有项目
│   ├── cleanup.sh                 # 清理脚本
│   ├── backup.sh                  # 备份脚本
│   └── utils/                     # 工具脚本
│       ├── check-deps.sh          # 检查依赖
│       ├── generate-docs.sh       # 生成文档
│       └── validate.sh            # 验证代码

# 配置文件目录
├── config/                          # 配置文件
│   ├── solana/                     # Solana 配置
│   │   ├── localnet.yml           # 本地网络配置
│   │   └── devnet.yml             # 开发网络配置
│   ├── substrate/                  # Substrate 配置
│   │   ├── dev.yaml               # 开发链配置
│   │   ├── testnet.yaml           # 测试网配置
│   │   └── chainspec.json         # 链规范
│   └── shared/                     # 共享配置
│       ├── network.json           # 网络配置
│       └── constants.json         # 常量定义

# 示例和演示目录
├── examples/                        # 示例代码
│   ├── solana-client/              # Solana 客户端示例
│   │   ├── create-token.js         # 创建代币示例
│   │   ├── mint-tokens.js         # 铸造代币示例
│   │   └── transfer-tokens.js     # 转移代币示例
│   ├── substrate-client/           # Substrate 客户端示例
│   │   ├── create-proposal.js      # 创建提案示例
│   │   ├── vote.js                # 投票示例
│   │   └── query-state.js         # 查询状态示例
│   └── integration/                # 集成示例
│       ├── cross-chain.js         # 跨链交互示例
│       └── full-workflow.js       # 完整工作流示例

# 测试数据目录
├── test-data/                       # 测试数据
│   ├── fixtures/                   # 测试夹具
│   ├── keys/                       # 测试密钥
│   │   ├── solana/                # Solana 测试密钥
│   │   └── substrate/             # Substrate 测试密钥
│   └── states/                     # 预设状态数据

# 部署相关目录
├── deployment/                      # 部署相关文件
│   ├── docker/                     # Docker 配置
│   │   ├── Dockerfile.solana      # Solana Docker 镜像
│   │   ├── Dockerfile.substrate   # Substrate Docker 镜像
│   │   └── docker-compose.yml     # 容器编排
│   ├── kubernetes/                 # Kubernetes 配置
│   │   ├── solana-deployment.yml  # Solana 部署
│   │   └── substrate-deployment.yml # Substrate 部署
│   ├── terraform/                  # Terraform 配置
│   │   ├── main.tf                # 主配置文件
│   │   ├── variables.tf           # 变量定义
│   │   └── outputs.tf             # 输出定义
│   └── ci-cd/                      # CI/CD 配置
│       ├── .github/               # GitHub Actions
│       │   └── workflows/
│       │       ├── test.yml       # 测试工作流
│       │       ├── build.yml      # 构建工作流
│       │       └── deploy.yml     # 部署工作流
│       └── gitlab-ci.yml          # GitLab CI 配置

# 开发工具目录
├── tools/                          # 开发工具
│   ├── generators/                 # 代码生成器
│   │   ├── pallet-generator.py    # Pallet 生成器
│   │   └── test-generator.py      # 测试生成器
│   ├── linters/                    # 代码检查工具
│   │   ├── .solhint.json          # Solana 代码检查
│   │   └── .rustfmt.toml          # Rust 代码格式化
│   └── analyzers/                  # 代码分析工具
│       ├── complexity-check.py    # 复杂度检查
│       └── security-scan.py       # 安全扫描

# 文档生成目录
├── docs-generated/                  # 自动生成的文档
│   ├── api/                        # API 文档
│   │   ├── solana/                # Solana API 文档
│   │   └── substrate/             # Substrate API 文档
│   ├── architecture/               # 架构文档
│   └── tutorials/                  # 教程文档

# 结果和输出目录
├── output/                          # 构建和测试输出
│   ├── builds/                     # 构建产物
│   │   ├── solana/                # Solana 构建结果
│   │   └── substrate/             # Substrate 构建结果
│   ├── tests/                      # 测试结果
│   │   ├── reports/               # 测试报告
│   │   └── coverage/              # 代码覆盖率
│   └── artifacts/                  # 部署产物
│       ├── program-id.json        # 程序 ID
│       └── genesis.json           # 创世块
```

## 文件命名规范

### 1. 通用规范
- 使用小写字母和连字符：`my-feature.rs`
- 测试文件以 `_test.rs` 或 `.test.ts` 结尾
- 配置文件使用标准名称：`Cargo.toml`, `package.json`
- 文档使用 Markdown 格式：`.md`

### 2. Rust 文件规范
```rust
// 库文件
lib.rs                    // 库入口
main.rs                   // 可执行程序入口
mod.rs                    // 模块入口

// 功能模块
account.rs                // 账户相关
token.rs                  // 代币相关
voting.rs                 // 投票相关
error.rs                  // 错误定义
event.rs                  // 事件定义
instruction.rs            // 指令定义
processor.rs              // 处理逻辑
state.rs                  // 状态管理
utils.rs                  // 工具函数

// 测试文件
mod_test.rs               // 模块测试
integration_test.rs       // 集成测试
benchmark_test.rs         // 性能测试
```

### 3. TypeScript/JavaScript 文件规范
```typescript
// 主要文件
index.ts                  // 入口文件
main.ts                   // 主程序
config.ts                 // 配置文件

// 功能模块
token.ts                  // 代币相关
proposal.ts               // 提案相关
vote.ts                   // 投票相关
client.ts                 // 客户端封装
utils.ts                  // 工具函数

// 测试文件
*.test.ts                 // 单元测试
*.spec.ts                 // 规格测试
integration.test.ts       // 集成测试
```

## 代码组织原则

### 1. 模块化设计
- 每个功能模块独立目录
- 清晰的依赖关系
- 最小化模块间耦合

### 2. 分层架构
```
presentation/             # 表示层（UI、CLI）
business/                 # 业务逻辑层
data/                     # 数据访问层
infrastructure/           # 基础设施层
```

### 3. 配置管理
- 环境特定配置
- 敏感信息分离
- 配置验证机制

## 版本控制策略

### 1. 分支策略
```
main                      # 主分支，稳定版本
develop                   # 开发分支
feature/*                 # 功能分支
hotfix/*                  # 热修复分支
release/*                 # 发布分支
```

### 2. 提交规范
```
feat: 新功能
fix: 修复
docs: 文档
style: 格式化
refactor: 重构
test: 测试
chore: 构建过程或辅助工具的变动
```

### 3. 标签管理
```
v1.0.0                    # 主版本
v1.1.0                    # 次版本
v1.1.1                    # 补丁版本
```

## 文档管理

### 1. 文档分类
- **用户文档**：安装、使用、配置
- **开发文档**：API、架构、设计
- **运维文档**：部署、监控、故障排除

### 2. 文档结构
```markdown
# 文档标题
## 概述
## 前置条件
## 快速开始
## 详细说明
## 示例代码
## 常见问题
## 参考资料
```

### 3. 自动化文档
- 代码注释生成 API 文档
- CI/CD 自动生成部署文档
- 测试覆盖率报告

## 质量保证

### 1. 代码检查
- 代码格式化
- 静态分析
- 安全扫描

### 2. 测试策略
- 单元测试：覆盖率 > 80%
- 集成测试：关键路径
- 端到端测试：用户场景

### 3. 性能监控
- 基准测试
- 内存使用监控
- 网络性能测试

## 部署和运维

### 1. 环境管理
```
development               # 开发环境
testing                   # 测试环境
staging                   # 预发布环境
production                # 生产环境
```

### 2. 监控和日志
- 应用性能监控
- 错误日志收集
- 系统资源监控

### 3. 备份和恢复
- 定期数据备份
- 灾难恢复计划
- 数据迁移工具

这个项目结构确保了：
1. **清晰的组织**：易于导航和维护
2. **模块化设计**：便于并行开发
3. **完整的文档**：降低学习成本
4. **自动化工具**：提高开发效率
5. **质量保证**：确保代码质量
6. **运维友好**：便于部署和维护