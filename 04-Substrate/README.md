# 多链合约开发教学作业

## 项目概述

本项目是一个综合性的多链合约开发作业，涵盖 Solana 智能合约开发和 Substrate 区块链框架开发。通过完成本作业，学生将深入理解两大主流区块链平台的开发模式和技术特点。

## 任务目标

- **理论学习**：深入理解 Sealevel 运行时、BPF 虚拟机、FRAME 框架和 Pallet 设计模式
- **实践开发**：实现完整的 SPL 代币程序和 Substrate 自定义区块链 Runtime
- **工程能力**：掌握多链开发的完整流程，包括设计、实现、测试、部署和维护

## 项目结构

```
task04/
├── README.md                    # 本文件 - 项目总体介绍
├── GUIDE.md                     # 详细任务指导
├── PROJECT_STRUCTURE.md         # 项目结构说明
├── TIMELINE.md                  # 时间规划和里程碑
├── TASK04.md                    # 原始作业要求
├── docs/                        # 详细文档目录
│   ├── solana-theory.md        # Solana 理论题答案
│   ├── substrate-theory.md     # Substrate 理论题答案
│   ├── solana-implementation-guide.md  # Solana 实现指南
│   └── substrate-implementation-guide.md # Substrate 实现指南
├── solana/                      # Solana SPL 代币项目
└── substrate/                   # Substrate 区块链项目
```

## 核心内容

### 第一部分：Solana 智能合约

#### 理论研究
1. **Sealevel 运行时分析**
   - 并行执行机制的优势
   - 与传统 EVM 的对比分析
   - 实际应用场景案例

2. **BPF 虚拟机研究**
   - 安全性特性分析
   - 可扩展性影响评估
   - 具体实现案例

#### 实践开发
- **SPL 代币发行程序**
  - 代币铸造功能
  - 代币转移功能
  - 余额查询功能
  - 权限控制机制

### 第二部分：Substrate 开发

#### 理论研究
1. **FRAME 框架分析**
   - 模块化开发优势
   - 开发效率提升分析
   - 与 balances 模块的深入分析

2. **Pallet 设计模式研究**
   - 设计模式原理
   - 与传统模块的差异对比
   - 自定义功能实现优势

#### 实践开发
- **自定义区块链 Runtime**
  - 基础账户管理
  - 交易处理机制
  - 区块生成功能
- **自定义投票 Pallet**
  - 提案创建和管理
  - 投票机制实现
  - 执行和结果处理

## 快速开始

### 1. 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"

# 安装 Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# 安装 Substrate 工具
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### 2. 项目初始化

```bash
# 克隆项目模板
git clone <项目地址>
cd task04

# 创建项目结构
mkdir -p solana substrate docs scripts

# 初始化 Solana 项目
cd solana
anchor init solana-spl-token
cd ..

# 初始化 Substrate 项目
git clone https://github.com/substrate-developer-hub/substrate-node-template substrate
```

### 3. 开发流程

按照 [TIMELINE.md](TIMELINE.md) 中的时间规划进行开发：

1. **第1-2周**：理论研究和环境准备
2. **第3-4周**：Solana 实践开发
3. **第5-6周**：Substrate 实践开发
4. **第7周**：整合和优化

## 详细指南

### 理论学习指南
- 参见 [docs/solana-theory.md](docs/solana-theory.md)
- 参见 [docs/substrate-theory.md](docs/substrate-theory.md)

### 实现指南
- Solana 实现：[docs/solana-implementation-guide.md](docs/solana-implementation-guide.md)
- Substrate 实现：[docs/substrate-implementation-guide.md](docs/substrate-implementation-guide.md)

### 项目结构
- 详细的项目组织：[PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)

### 时间规划
- 完整的开发计划：[TIMELINE.md](TIMELINE.md)

## 技术栈

### Solana 生态系统
- **编程语言**：Rust
- **开发框架**：Anchor
- **代币标准**：SPL Token
- **运行时**：Sealevel
- **虚拟机**：BPF

### Substrate 生态系统
- **编程语言**：Rust
- **开发框架**：FRAME
- **共识算法**：Aura + Grandpa
- **模块设计**：Pallet
- **区块链类型**：Solochain

## 评分标准

### 理论部分（40%）
- 概念理解准确性：15%
- 分析深度和逻辑性：15%
- 案例质量和相关性：10%

### 实践部分（60%）
- 代码质量和功能完整性：25%
- 文档详细程度和规范性：20%
- 创新性和扩展性：15%

## 学习资源

### 官方文档
- [Solana 开发者文档](https://docs.solana.com/)
- [Anchor 框架文档](https://anchor-lang.com/)
- [Substrate 开发者中心](https://substrate.io/developers/)
- [Polkadot Wiki](https://wiki.polkadot.network/)

### 技术社区
- Solana Discord：https://discord.gg/solana
- Substrate Element：https://matrix.to/#/#substrate-technical:matrix.org
- Stack Overflow：标签 `solana` 和 `substrate`

### 推荐教程
- Solana by Example
- Substrate How-to Guides
- Rust Programming Language

## 常见问题

### Q: 如何解决环境配置问题？
A: 仔细阅读各平台的安装指南，确保版本兼容性，使用容器化环境作为备选方案。

### Q: 开发过程中遇到技术难题怎么办？
A: 查阅官方文档，搜索相关社区讨论，在技术论坛提问，考虑简化实现方案。

### Q: 如何确保代码质量？
A: 编写详细的单元测试，进行代码审查，使用静态分析工具，遵循最佳实践。

### Q: 项目时间不够怎么办？
A: 优先实现核心功能，简化非关键特性，参考时间规划调整开发优先级。

## 贡献指南

### 提交规范
```
feat: 新功能
fix: 修复
docs: 文档
style: 格式化
refactor: 重构
test: 测试
chore: 构建/工具
```

### 分支策略
- `main`: 主分支，稳定版本
- `develop`: 开发分支
- `feature/*`: 功能分支
- `hotfix/*`: 热修复分支

## 联系方式

如有问题或建议，请通过以下方式联系：

- GitHub Issues：提交问题和建议
- 技术论坛：参与技术讨论
- 邮件联系：发送技术咨询邮件

## 许可证

本项目采用 MIT 许可证，详见 [LICENSE](LICENSE) 文件。

---

## 快速导航

| 任务 | 文档链接 | 状态 |
|------|----------|------|
| 总体指导 | [GUIDE.md](GUIDE.md) | 📋 待开始 |
| 项目结构 | [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) | 📋 待开始 |
| 时间规划 | [TIMELINE.md](TIMELINE.md) | 📋 待开始 |
| Solana 理论 | [docs/solana-theory.md](docs/solana-theory.md) | 📋 待开始 |
| Solana 实现 | [docs/solana-implementation-guide.md](docs/solana-implementation-guide.md) | 📋 待开始 |
| Substrate 理论 | [docs/substrate-theory.md](docs/substrate-theory.md) | 📋 待开始 |
| Substrate 实现 | [docs/substrate-implementation-guide.md](docs/substrate-implementation-guide.md) | 📋 待开始 |

---

**开始你的多链合约开发之旅吧！🚀**

记住：这个项目不仅是为了完成作业，更是为了掌握未来区块链开发的核心技能。祝你学习顺利！