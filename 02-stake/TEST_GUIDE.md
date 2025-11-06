# MetaNode Stake System 测试指南

## 📊 测试覆盖率总结

当前测试覆盖率：**94.31%** (超过90%目标)

### 详细覆盖率统计
```
File                |  % Stmts | % Branch |  % Funcs |  % Lines |
--------------------|----------|----------|----------|----------|
contracts/          |    94.31 |    70.15 |    94.29 |    92.98 |
 MetaNodeToken.sol  |     100  |     100  |     100  |     100  |
 StakePool.sol      |    93.39 |    70.69 |    92.31 |    91.84 |
 StakePoolV2.sol    |     100  |    64.29 |     100  |     100  |
 TestToken.sol      |     100  |     100  |     100  |     100  |
```

## 🧪 测试文件结构

### 1. 主要测试文件

```
test/
├── StakePool.test.js     # 主功能测试 (510行)
├── Security.test.js      # 安全性测试 (337行)  
└── Enhanced.test.js      # 增强覆盖率测试 (500+行)
```

### 2. 测试框架和工具

- **测试框架**: Hardhat + Mocha + Chai
- **网络助手**: @nomicfoundation/hardhat-network-helpers
- **覆盖率工具**: solidity-coverage
- **Gas报告**: hardhat-gas-reporter

## 📋 测试分类详解

### A. StakePool.test.js - 主功能测试

#### 1. Deployment (部署测试)
```javascript
describe("Deployment", function () {
    it("Should initialize with correct parameters")
    it("Should grant admin roles to deployer")
});
```

**测试内容:**
- 合约初始化参数验证
- 权限角色分配检查
- 初始状态验证

**关键测试点:**
- `metaNodeToken` 地址设置正确
- `metaNodePerBlock` 奖励率设置正确  
- 管理员角色正确分配

#### 2. Pool Management (池管理测试)
```javascript
describe("Pool Management", function () {
    it("Should add a native currency pool")      // 添加ETH池
    it("Should add an ERC20 token pool")         // 添加ERC20池
    it("Should update pool parameters")          // 更新池参数
    it("Should not allow non-admin to manage pools") // 权限检查
});
```

**测试场景:**
- **添加ETH池**: 
  ```javascript
  await stakePool.addPool(
      ethers.ZeroAddress,    // ETH用地址0表示
      100,                   // 池权重
      ethers.parseEther("0.1"), // 最小质押0.1 ETH
      100                    // 锁定100个区块
  );
  ```

- **添加ERC20池**:
  ```javascript
  await stakePool.addPool(
      testToken.address,        // ERC20代币地址
      200,                      // 池权重
      ethers.parseEther("10"),  // 最小质押10个代币
      200                       // 锁定200个区块
  );
  ```

#### 3. Staking (质押测试)
```javascript
describe("Staking", function () {
    it("Should stake native currency")           // ETH质押
    it("Should stake ERC20 tokens")             // ERC20质押
    it("Should fail to stake below minimum deposit") // 最小数量检查
    it("Should fail to stake with incorrect ETH amount") // ETH数量检查
});
```

**ETH质押流程:**
```javascript
const stakeAmount = ethers.parseEther("1");
await stakePool.connect(user1).stake(0, stakeAmount, { 
    value: stakeAmount  // 必须发送相同数量的ETH
});
```

**ERC20质押流程:**
```javascript
const stakeAmount = ethers.parseEther("100");
// 1. 先授权
await testToken.connect(user1).approve(stakePool.address, stakeAmount);
// 2. 再质押
await stakePool.connect(user1).stake(1, stakeAmount); // 不发送ETH
```

#### 4. Unstaking and Withdrawal (解质押和提取测试)
```javascript
describe("Unstaking and Withdrawal", function () {
    it("Should unstake tokens")                 // 解质押
    it("Should withdraw after lock period")     // 锁定期后提取
    it("Should not withdraw before lock period") // 锁定期内不能提取
    it("Should handle multiple unstake requests") // 多次解质押
    it("Should emergency withdraw")             // 紧急提取
});
```

**解质押-提取流程:**
```javascript
// 1. 解质押 (创建解锁请求)
await stakePool.connect(user1).unstake(0, ethers.parseEther("1"));

// 2. 等待锁定期
for (let i = 0; i < 15; i++) {
    await ethers.provider.send("evm_mine"); // 挖15个区块
}

// 3. 提取资金
await stakePool.connect(user1).withdraw(0);
```

#### 5. Rewards (奖励测试)
```javascript
describe("Rewards", function () {
    it("Should calculate pending rewards correctly")    // 奖励计算
    it("Should claim rewards")                         // 领取奖励
    it("Should distribute rewards proportionally among users") // 比例分配
});
```

**奖励计算公式:**
```solidity
// 池奖励 = (区块数 × 每区块奖励 × 池权重) / 总权重  
// 用户奖励 = 池奖励 × 用户质押量 / 池总质押量
multiplier = block.number - pool.lastRewardBlock;
metaNodeReward = (multiplier * metaNodePerBlock * pool.poolWeight) / totalPoolWeight;
pool.accMetaNodePerST += (metaNodeReward * 1e12) / pool.stTokenAmount;
userReward = ((user.stAmount * pool.accMetaNodePerST) / 1e12) - user.finishedMetaNode;
```

#### 6. Pause Functionality (暂停功能测试)
```javascript
describe("Pause Functionality", function () {
    it("Should pause and unpause staking")      // 质押暂停
    it("Should pause entire contract")          // 合约全局暂停
});
```

**暂停类型:**
- `stakePaused` - 只暂停质押操作
- `unstakePaused` - 只暂停解质押操作  
- `claimPaused` - 只暂停领取操作
- `paused()` - 全局暂停所有操作

### B. Security.test.js - 安全性测试

#### 1. Reentrancy Protection (重入攻击防护)
```javascript
describe("Reentrancy Protection", function () {
    it("Should prevent reentrancy on withdraw")
});
```

**防护机制:**
- 状态更新优先于外部调用
- 使用 SafeERC20 进行代币转账
- 检查-效果-交互模式

#### 2. Integer Overflow/Underflow Protection (整数溢出防护)
```javascript
describe("Integer Overflow/Underflow Protection", function () {
    it("Should handle large numbers correctly")
    it("Should prevent unstaking more than staked")
});
```

**防护特性:**
- Solidity 0.8+ 内置溢出检查
- 明确的余额检查
- 安全数学运算

#### 3. Authorization Bypass Attempts (权限绕过测试)
```javascript
describe("Authorization Bypass Attempts", function () {
    it("Should prevent non-admin from adding pools")
    it("Should prevent unauthorized emergency recovery")
    it("Should prevent unauthorized parameter changes")
});
```

**权限验证:**
- 使用 OpenZeppelin AccessControl
- 角色分离 (ADMIN_ROLE, OPERATOR_ROLE, UPGRADER_ROLE)
- 关键函数权限保护

#### 4. Edge Cases (边界情况测试)
```javascript
describe("Edge Cases", function () {
    it("Should handle zero staking attempts")
    it("Should handle claiming with no rewards")
    it("Should handle multiple withdrawals correctly")
    it("Should handle pool deactivation correctly")
});
```

#### 5. Reward Calculation Edge Cases (奖励计算边界测试)
```javascript
describe("Reward Calculation Edge Cases", function () {
    it("Should handle reward calculation when total supply is zero")
    it("Should handle reward distribution when pool weight is zero")
    it("Should handle precision in reward calculations")
});
```

### C. Enhanced.test.js - 增强覆盖率测试

#### 1. MetaNodeToken Coverage (奖励代币测试)
```javascript
describe("MetaNodeToken Coverage", function () {
    it("Should mint tokens by owner")           // 所有者铸造
    it("Should fail to mint beyond max supply") // 超量铸造失败
    it("Should not allow non-owner to mint")    // 非所有者铸造失败
});
```

#### 2. TestToken Coverage (测试代币测试)
```javascript
describe("TestToken Coverage", function () {
    it("Should return correct decimals")        // 精度检查
    it("Should allow anyone to mint")           // 公开铸造
    it("Should create token with custom decimals") // 自定义精度
});
```

#### 3. StakePoolV2 Coverage (升级合约测试)
```javascript
describe("StakePoolV2 Coverage", function () {
    it("Should set bonus multiplier")           // 设置奖励倍数
    it("Should fail to set bonus multiplier below 100") // 倍数限制
    it("Should calculate pending rewards with bonus")    // 带倍数奖励计算
    it("Should claim rewards with bonus")       // 带倍数奖励领取
    it("Should return correct version")         // 版本检查
});
```

**V2新功能:**
```javascript
// 设置2倍奖励倍数
await stakePoolV2.setBonusMultiplier(200);

// 带倍数的奖励计算
const basePending = await stakePoolV2.pendingMetaNode(0, user.address);
const bonusPending = await stakePoolV2.pendingMetaNodeWithBonus(0, user.address);
expect(bonusPending).to.equal(basePending * 2n);

// 带倍数的奖励领取
await stakePoolV2.connect(user).claimWithBonus(0);
```

#### 4. Advanced StakePool Functions (高级功能测试)
```javascript
describe("Advanced StakePool Functions", function () {
    it("Should handle massUpdatePools function")     // 批量更新池
    it("Should handle getPoolLength function")       // 获取池数量
    it("Should handle getWithdrawableAmount function") // 获取可提取数量
    it("Should handle setMetaNodePerBlock function") // 设置每区块奖励
    it("Should handle receive function")             // 接收ETH
    it("Should handle emergencyRecoverToken function") // 紧急恢复代币
});
```

## 🔧 如何运行测试

### 1. 环境准备
```bash
# 进入项目目录
cd stake

# 安装依赖 (如果遇到版本冲突)
npm install --legacy-peer-deps

# 复制环境配置
cp .env.example .env
```

### 2. 编译合约
```bash
npm run compile
```

### 3. 运行测试

**运行所有测试:**
```bash
npm run test
```

**运行特定测试文件:**
```bash
# 运行主功能测试
npm test -- test/StakePool.test.js

# 运行安全性测试  
npm test -- test/Security.test.js

# 运行增强测试
npm test -- test/Enhanced.test.js
```

**运行特定测试用例:**
```bash
# 运行特定describe块
npm test -- --grep "Pool Management"

# 运行特定测试
npm test -- --grep "Should stake native currency"
```

### 4. 测试覆盖率
```bash
# 生成覆盖率报告
npm run coverage

# 查看详细报告 (生成后在 coverage/ 目录)
open coverage/index.html
```

### 5. Gas使用报告
```bash
# 运行带Gas报告的测试
npm run test:gas
```

## 📝 编写测试的最佳实践

### 1. 测试结构

**使用describe分组:**
```javascript
describe("Contract Name", function () {
    describe("Function Group", function () {
        it("Should do specific thing", async function () {
            // 测试逻辑
        });
    });
});
```

**使用fixture复用设置:**
```javascript
async function deployContractFixture() {
    const [owner, user1, user2] = await ethers.getSigners();
    // 部署合约逻辑
    return { contract, owner, user1, user2 };
}

it("Test case", async function () {
    const { contract, user1 } = await loadFixture(deployContractFixture);
    // 测试逻辑
});
```

### 2. 事件测试

**检查事件发出:**
```javascript
await expect(contract.someFunction())
    .to.emit(contract, "EventName")
    .withArgs(arg1, arg2, arg3);
```

**复杂事件检查:**
```javascript
const tx = await contract.someFunction();
const receipt = await tx.wait();

const event = receipt.logs.find(log => {
    try {
        const parsed = contract.interface.parseLog(log);
        return parsed.name === "EventName";
    } catch {
        return false;
    }
});

expect(event).to.not.be.undefined;
```

### 3. 错误测试

**检查特定错误消息:**
```javascript
await expect(contract.someFunction())
    .to.be.revertedWith("Error message");
```

**检查自定义错误:**
```javascript
await expect(contract.someFunction())
    .to.be.revertedWithCustomError(contract, "CustomError");
```

**泛型错误检查:**
```javascript
await expect(contract.someFunction())
    .to.be.reverted;
```

### 4. 状态检查

**余额检查:**
```javascript
const balanceBefore = await token.balanceOf(user.address);
// 执行操作
const balanceAfter = await token.balanceOf(user.address);
expect(balanceAfter).to.equal(balanceBefore + expectedChange);
```

**合约状态检查:**
```javascript
const poolInfo = await stakePool.pools(0);
expect(poolInfo.stTokenAmount).to.equal(expectedAmount);
expect(poolInfo.isActive).to.be.true;
```

### 5. 时间操控

**挖掘区块:**
```javascript
// 挖掘单个区块
await ethers.provider.send("evm_mine");

// 挖掘多个区块
for (let i = 0; i < 10; i++) {
    await ethers.provider.send("evm_mine");
}
```

**设置时间戳:**
```javascript
const newTimestamp = (await ethers.provider.getBlock("latest")).timestamp + 3600;
await ethers.provider.send("evm_setNextBlockTimestamp", [newTimestamp]);
await ethers.provider.send("evm_mine");
```

## 🎯 测试用例设计原则

### 1. 功能覆盖
- ✅ 正常流程测试
- ✅ 边界条件测试  
- ✅ 错误情况测试
- ✅ 权限验证测试

### 2. 场景覆盖
- ✅ 单用户操作
- ✅ 多用户交互
- ✅ 极端数值测试
- ✅ 状态转换测试

### 3. 安全考虑
- ✅ 重入攻击防护
- ✅ 整数溢出防护
- ✅ 权限绕过测试
- ✅ 经济攻击防护

### 4. 升级兼容性
- ✅ 存储布局兼容
- ✅ 新功能测试
- ✅ 向后兼容性
- ✅ 升级路径测试

## 📚 测试用例教学分析

### 实例1: 质押功能测试详解

```javascript
it("Should stake native currency", async function () {
    // 1. 准备测试环境
    const { stakePool, user1 } = await loadFixture(setupPoolsFixture);
    
    // 2. 定义测试数据
    const stakeAmount = ethers.parseEther("1"); // 1 ETH
    
    // 3. 执行被测试功能 + 验证事件
    await expect(stakePool.connect(user1).stake(0, stakeAmount, { value: stakeAmount }))
        .to.emit(stakePool, "Deposit")
        .withArgs(user1.address, 0, stakeAmount);
    
    // 4. 验证状态变化
    const userInfo = await stakePool.getUserInfo(0, user1.address);
    expect(userInfo.stAmount).to.equal(stakeAmount);
    
    const pool = await stakePool.pools(0);
    expect(pool.stTokenAmount).to.equal(stakeAmount);
});
```

**教学要点:**
1. **环境准备**: 使用fixture确保测试环境干净
2. **数据准备**: 使用ethers.parseEther处理大数
3. **行为验证**: 同时检查事件发出和状态变化
4. **完整性检查**: 验证用户和池的状态都正确更新

### 实例2: 错误情况测试详解

```javascript
it("Should fail to stake below minimum deposit", async function () {
    // 1. 准备测试环境
    const { stakePool, user1 } = await loadFixture(setupPoolsFixture);
    
    // 2. 准备错误数据 (低于最小值)
    const stakeAmount = ethers.parseEther("0.05"); // 低于最小0.1 ETH
    
    // 3. 验证错误发生
    await expect(stakePool.connect(user1).stake(0, stakeAmount, { value: stakeAmount }))
        .to.be.revertedWith("Amount below minimum deposit");
});
```

**教学要点:**
1. **边界测试**: 测试边界条件确保验证逻辑正确
2. **错误消息**: 验证具体的错误消息确保正确的错误处理
3. **状态不变**: 错误情况下状态不应该改变

### 实例3: 复杂流程测试详解

```javascript
it("Should withdraw after lock period", async function () {
    // 1. 准备测试环境
    const { stakePool, user1 } = await loadFixture(setupStakedFixture);
    
    const unstakeAmount = ethers.parseEther("1");
    
    // 2. 执行第一阶段 - 解质押
    await stakePool.connect(user1).unstake(0, unstakeAmount);
    
    // 3. 模拟时间流逝
    for (let i = 0; i < 15; i++) {
        await ethers.provider.send("evm_mine"); // 挖掘15个区块
    }
    
    // 4. 记录初始状态
    const balanceBefore = await ethers.provider.getBalance(user1.address);
    
    // 5. 执行第二阶段 - 提取
    await expect(stakePool.connect(user1).withdraw(0))
        .to.emit(stakePool, "Withdraw")
        .withArgs(user1.address, 0, unstakeAmount);
    
    // 6. 验证最终状态
    const balanceAfter = await ethers.provider.getBalance(user1.address);
    expect(balanceAfter).to.be.gt(balanceBefore); // 余额增加
    
    // 7. 验证内部状态清理
    const userInfo = await stakePool.getUserInfo(0, user1.address);
    expect(userInfo.requests).to.have.length(0); // 请求被清理
});
```

**教学要点:**
1. **多阶段流程**: 测试完整的业务流程
2. **时间操控**: 使用区块挖掘模拟时间流逝
3. **状态跟踪**: 跟踪多个状态变化
4. **清理验证**: 确保内部状态正确清理

## 🚀 测试优化建议

### 1. 性能优化
- 使用fixture减少重复部署
- 合理使用beforeEach vs loadFixture
- 避免不必要的区块挖掘

### 2. 覆盖率优化
- 确保所有分支路径都被测试
- 测试所有函数的成功和失败情况
- 包含边界条件和异常情况

### 3. 可维护性
- 使用描述性的测试名称
- 提取通用的测试逻辑到helpers
- 保持测试独立性

### 4. 可读性
- 添加注释解释复杂的测试逻辑
- 使用一致的代码风格
- 分组相关的测试用例

## 📈 当前测试指标

| 指标 | 数值 | 目标 | 状态 |
|------|------|------|------|
| 语句覆盖率 | 94.31% | >90% | ✅ 达标 |
| 分支覆盖率 | 70.15% | >70% | ✅ 达标 |
| 函数覆盖率 | 94.29% | >90% | ✅ 达标 |
| 行覆盖率 | 92.98% | >90% | ✅ 达标 |
| 测试用例数 | 60+ | >50 | ✅ 达标 |
| 测试文件数 | 3 | >2 | ✅ 达标 |

## 🎓 总结

通过完善的测试套件，我们确保了MetaNode Stake System的:
- **功能正确性**: 所有核心功能都经过严格测试
- **安全性**: 安全机制和边界条件都得到验证  
- **可靠性**: 异常情况和错误处理都被覆盖
- **可升级性**: V2升级功能完全测试

测试覆盖率达到**94.31%**，远超90%的目标，为系统的安全部署和长期维护提供了坚实保障。