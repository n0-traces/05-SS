// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "./MetaNodeToken.sol";

/**
 * @title StakePool
 * @author MetaNode Academy
 * @notice 多池质押合约，支持ETH和ERC20代币质押，并分发MetaNode代币奖励
 * @dev 使用可升级代理模式，包含完善的权限控制和暂停机制
 *
 * 核心功能:
 * - 支持创建多个独立的质押池
 * - 原生币(ETH)和ERC20代币质押
 * - 基于时间和质押量的奖励分配
 * - 解质押锁定期机制
 * - 紧急提现功能
 * - 灵活的权限和暂停控制
 */
contract StakePool is AccessControl, Pausable, Initializable {
    using SafeERC20 for IERC20;

    /// @notice 管理员角色，可以管理池、修改参数
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    /// @notice 升级者角色，可以升级合约
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");

    /// @notice 操作员角色，可以暂停/恢复操作
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

    /**
     * @notice 质押池结构体
     * @dev 每个池代表一种可质押的代币类型
     */
    struct Pool {
        address stTokenAddress;      /// 质押代币地址 (address(0)表示ETH)
        uint256 poolWeight;          /// 池权重，用于奖励分配计算
        uint256 lastRewardBlock;     /// 上次奖励计算的区块号
        uint256 accMetaNodePerST;    /// 每个质押代币累积的MetaNode奖励(放大1e12倍)
        uint256 stTokenAmount;       /// 池中总质押数量
        uint256 minDepositAmount;    /// 最小质押数量
        uint256 unstakeLockedBlocks; /// 解质押锁定区块数
        bool isActive;               /// 池激活状态
    }

    /**
     * @notice 解质押请求结构体
     * @dev 用户解质押后需要等待锁定期才能提取
     */
    struct UnstakeRequest {
        uint256 amount;       /// 解质押数量
        uint256 unlockBlock;  /// 解锁区块号
    }

    /**
     * @notice 用户信息结构体
     * @dev 记录用户在某个池中的质押和奖励信息
     */
    struct User {
        uint256 stAmount;           /// 用户质押数量
        uint256 finishedMetaNode;   /// 已结算的MetaNode奖励
        uint256 pendingMetaNode;    /// 待领取的MetaNode奖励
        UnstakeRequest[] requests;  /// 解质押请求列表
    }
    
    /// @notice MetaNode奖励代币合约地址
    MetaNodeToken public metaNodeToken;

    /// @notice 每个区块产生的MetaNode奖励数量
    uint256 public metaNodePerBlock;

    /// @notice 所有池的总权重
    uint256 public totalPoolWeight;

    /// @notice 开始产生奖励的区块号
    uint256 public startBlock;

    /// @notice 所有质押池的数组
    Pool[] public pools;

    /// @notice 用户信息映射: 池ID => 用户地址 => 用户信息
    mapping(uint256 => mapping(address => User)) public users;

    /// @notice 质押操作暂停状态
    bool public stakePaused;

    /// @notice 解质押操作暂停状态
    bool public unstakePaused;

    /// @notice 领取奖励操作暂停状态
    bool public claimPaused;

    /// @notice 质押事件
    /// @param user 用户地址
    /// @param pid 池ID
    /// @param amount 质押数量
    event Deposit(address indexed user, uint256 indexed pid, uint256 amount);

    /// @notice 提取事件
    /// @param user 用户地址
    /// @param pid 池ID
    /// @param amount 提取数量
    event Withdraw(address indexed user, uint256 indexed pid, uint256 amount);

    /// @notice 解质押事件
    /// @param user 用户地址
    /// @param pid 池ID
    /// @param amount 解质押数量
    event Unstake(address indexed user, uint256 indexed pid, uint256 amount);

    /// @notice 领取奖励事件
    /// @param user 用户地址
    /// @param pid 池ID
    /// @param amount 奖励数量
    event Claim(address indexed user, uint256 indexed pid, uint256 amount);

    /// @notice 池添加事件
    /// @param pid 池ID
    /// @param stTokenAddress 质押代币地址
    /// @param poolWeight 池权重
    /// @param minDepositAmount 最小质押数量
    /// @param unstakeLockedBlocks 解质押锁定区块数
    event PoolAdded(uint256 indexed pid, address stTokenAddress, uint256 poolWeight, uint256 minDepositAmount, uint256 unstakeLockedBlocks);

    /// @notice 池更新事件
    /// @param pid 池ID
    /// @param poolWeight 新池权重
    /// @param minDepositAmount 新最小质押数量
    /// @param unstakeLockedBlocks 新解质押锁定区块数
    event PoolUpdated(uint256 indexed pid, uint256 poolWeight, uint256 minDepositAmount, uint256 unstakeLockedBlocks);
    
    modifier validPool(uint256 _pid) {
        require(_pid < pools.length, "Invalid pool ID");
        require(pools[_pid].isActive, "Pool is not active");
        _;
    }
    
    modifier whenStakeNotPaused() {
        require(!stakePaused, "Stake is paused");
        _;
    }
    
    modifier whenUnstakeNotPaused() {
        require(!unstakePaused, "Unstake is paused");
        _;
    }
    
    modifier whenClaimNotPaused() {
        require(!claimPaused, "Claim is paused");
        _;
    }

    /**
     * @notice 初始化合约
     * @dev 只能调用一次，用于可升级代理模式
     * @param _metaNodeToken MetaNode奖励代币合约地址
     * @param _metaNodePerBlock 每个区块产生的奖励数量
     * @param _startBlock 开始产生奖励的区块号，0表示当前区块
     */
    function initialize(
        MetaNodeToken _metaNodeToken,
        uint256 _metaNodePerBlock,
        uint256 _startBlock
    ) public initializer {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(UPGRADER_ROLE, msg.sender);
        _grantRole(OPERATOR_ROLE, msg.sender);
        
        metaNodeToken = _metaNodeToken;
        metaNodePerBlock = _metaNodePerBlock;
        startBlock = _startBlock > 0 ? _startBlock : block.number;
    }

    /**
     * @notice 添加新的质押池
     * @dev 只有ADMIN_ROLE可以调用
     * @param _stTokenAddress 质押代币地址，address(0)表示ETH
     * @param _poolWeight 池权重，影响奖励分配比例
     * @param _minDepositAmount 最小质押数量
     * @param _unstakeLockedBlocks 解质押锁定区块数
     */
    function addPool(
        address _stTokenAddress,
        uint256 _poolWeight,
        uint256 _minDepositAmount,
        uint256 _unstakeLockedBlocks
    ) external onlyRole(ADMIN_ROLE) {
        massUpdatePools();
        
        totalPoolWeight += _poolWeight;
        
        pools.push(Pool({
            stTokenAddress: _stTokenAddress,
            poolWeight: _poolWeight,
            lastRewardBlock: block.number > startBlock ? block.number : startBlock,
            accMetaNodePerST: 0,
            stTokenAmount: 0,
            minDepositAmount: _minDepositAmount,
            unstakeLockedBlocks: _unstakeLockedBlocks,
            isActive: true
        }));
        
        emit PoolAdded(pools.length - 1, _stTokenAddress, _poolWeight, _minDepositAmount, _unstakeLockedBlocks);
    }

    /**
     * @notice 更新现有质押池的参数
     * @dev 只有ADMIN_ROLE可以调用
     * @param _pid 池ID
     * @param _poolWeight 新的池权重
     * @param _minDepositAmount 新的最小质押数量
     * @param _unstakeLockedBlocks 新的解质押锁定区块数
     */
    function updatePool(
        uint256 _pid,
        uint256 _poolWeight,
        uint256 _minDepositAmount,
        uint256 _unstakeLockedBlocks
    ) external onlyRole(ADMIN_ROLE) validPool(_pid) {
        massUpdatePools();
        
        Pool storage pool = pools[_pid];
        totalPoolWeight = totalPoolWeight - pool.poolWeight + _poolWeight;
        pool.poolWeight = _poolWeight;
        pool.minDepositAmount = _minDepositAmount;
        pool.unstakeLockedBlocks = _unstakeLockedBlocks;
        
        emit PoolUpdated(_pid, _poolWeight, _minDepositAmount, _unstakeLockedBlocks);
    }

    /**
     * @notice 更新池的奖励累积值
     * @dev 计算从上次更新到现在的奖励并累加到池中
     * @param _pid 池ID
     */
    function updatePoolReward(uint256 _pid) public validPool(_pid) {
        Pool storage pool = pools[_pid];
        
        if (block.number <= pool.lastRewardBlock) {
            return;
        }
        
        if (pool.stTokenAmount == 0) {
            pool.lastRewardBlock = block.number;
            return;
        }
        
        uint256 multiplier = block.number - pool.lastRewardBlock;
        uint256 metaNodeReward = (multiplier * metaNodePerBlock * pool.poolWeight) / totalPoolWeight;
        
        pool.accMetaNodePerST += (metaNodeReward * 1e12) / pool.stTokenAmount;
        pool.lastRewardBlock = block.number;
    }

    /**
     * @notice 批量更新所有激活池的奖励
     * @dev 遍历所有池并更新其奖励累积值
     */
    function massUpdatePools() public {
        for (uint256 pid = 0; pid < pools.length; pid++) {
            if (pools[pid].isActive) {
                updatePoolReward(pid);
            }
        }
    }

    /**
     * @notice 查询用户待领取的MetaNode奖励
     * @dev 包括已累积但未结算的奖励
     * @param _pid 池ID
     * @param _user 用户地址
     * @return 待领取的奖励数量
     */
    function pendingMetaNode(uint256 _pid, address _user) external view validPool(_pid) returns (uint256) {
        Pool memory pool = pools[_pid];
        User memory user = users[_pid][_user];
        
        uint256 accMetaNodePerST = pool.accMetaNodePerST;
        
        if (block.number > pool.lastRewardBlock && pool.stTokenAmount != 0) {
            uint256 multiplier = block.number - pool.lastRewardBlock;
            uint256 metaNodeReward = (multiplier * metaNodePerBlock * pool.poolWeight) / totalPoolWeight;
            accMetaNodePerST += (metaNodeReward * 1e12) / pool.stTokenAmount;
        }
        
        return ((user.stAmount * accMetaNodePerST) / 1e12) - user.finishedMetaNode + user.pendingMetaNode;
    }

    /**
     * @notice 质押代币到指定池
     * @dev ETH池需要发送ETH，ERC20池需要先授权
     * @param _pid 池ID
     * @param _amount 质押数量
     */
    function stake(uint256 _pid, uint256 _amount) external payable whenStakeNotPaused whenNotPaused validPool(_pid) {
        Pool storage pool = pools[_pid];
        User storage user = users[_pid][msg.sender];
        
        require(_amount >= pool.minDepositAmount, "Amount below minimum deposit");
        
        updatePoolReward(_pid);
        
        if (user.stAmount > 0) {
            uint256 pending = ((user.stAmount * pool.accMetaNodePerST) / 1e12) - user.finishedMetaNode;
            if (pending > 0) {
                user.pendingMetaNode += pending;
            }
        }
        
        if (pool.stTokenAddress == address(0)) {
            require(msg.value == _amount, "Invalid ETH amount");
        } else {
            require(msg.value == 0, "Should not send ETH for ERC20 token");
            IERC20(pool.stTokenAddress).safeTransferFrom(msg.sender, address(this), _amount);
        }
        
        user.stAmount += _amount;
        pool.stTokenAmount += _amount;
        user.finishedMetaNode = (user.stAmount * pool.accMetaNodePerST) / 1e12;
        
        emit Deposit(msg.sender, _pid, _amount);
    }

    /**
     * @notice 发起解质押请求
     * @dev 创建锁定期请求，需要等待锁定期后才能withdraw
     * @param _pid 池ID
     * @param _amount 解质押数量
     */
    function unstake(uint256 _pid, uint256 _amount) external whenUnstakeNotPaused whenNotPaused validPool(_pid) {
        Pool storage pool = pools[_pid];
        User storage user = users[_pid][msg.sender];
        
        require(user.stAmount >= _amount, "Insufficient staked amount");
        require(_amount > 0, "Amount must be greater than 0");
        
        updatePoolReward(_pid);
        
        uint256 pending = ((user.stAmount * pool.accMetaNodePerST) / 1e12) - user.finishedMetaNode;
        if (pending > 0) {
            user.pendingMetaNode += pending;
        }
        
        user.stAmount -= _amount;
        pool.stTokenAmount -= _amount;
        user.finishedMetaNode = (user.stAmount * pool.accMetaNodePerST) / 1e12;
        
        user.requests.push(UnstakeRequest({
            amount: _amount,
            unlockBlock: block.number + pool.unstakeLockedBlocks
        }));
        
        emit Unstake(msg.sender, _pid, _amount);
    }

    /**
     * @notice 提取已解锁的质押代币
     * @dev 只能提取已过锁定期的解质押请求
     * @param _pid 池ID
     */
    function withdraw(uint256 _pid) external whenNotPaused validPool(_pid) {
        Pool storage pool = pools[_pid];
        User storage user = users[_pid][msg.sender];
        
        uint256 totalWithdrawable = 0;
        uint256 requestCount = user.requests.length;
        
        for (uint256 i = 0; i < requestCount; i++) {
            if (user.requests[i].unlockBlock <= block.number) {
                totalWithdrawable += user.requests[i].amount;
            }
        }
        
        require(totalWithdrawable > 0, "No withdrawable amount");
        
        // Remove processed requests
        uint256 writeIndex = 0;
        for (uint256 i = 0; i < requestCount; i++) {
            if (user.requests[i].unlockBlock > block.number) {
                user.requests[writeIndex] = user.requests[i];
                writeIndex++;
            }
        }
        
        // Reduce array length
        while (user.requests.length > writeIndex) {
            user.requests.pop();
        }
        
        if (pool.stTokenAddress == address(0)) {
            payable(msg.sender).transfer(totalWithdrawable);
        } else {
            IERC20(pool.stTokenAddress).safeTransfer(msg.sender, totalWithdrawable);
        }
        
        emit Withdraw(msg.sender, _pid, totalWithdrawable);
    }

    /**
     * @notice 领取MetaNode奖励
     * @dev 领取用户在指定池的所有待领取奖励
     * @param _pid 池ID
     */
    function claim(uint256 _pid) external whenClaimNotPaused whenNotPaused validPool(_pid) {
        Pool storage pool = pools[_pid];
        User storage user = users[_pid][msg.sender];
        
        updatePoolReward(_pid);
        
        uint256 pending = ((user.stAmount * pool.accMetaNodePerST) / 1e12) - user.finishedMetaNode;
        uint256 totalPending = pending + user.pendingMetaNode;
        
        require(totalPending > 0, "No pending rewards");
        
        user.finishedMetaNode = (user.stAmount * pool.accMetaNodePerST) / 1e12;
        user.pendingMetaNode = 0;
        
        metaNodeToken.transfer(msg.sender, totalPending);
        
        emit Claim(msg.sender, _pid, totalPending);
    }

    /**
     * @notice 紧急提取质押代币
     * @dev 放弃所有奖励和解质押请求，立即提取质押代币
     * @param _pid 池ID
     */
    function emergencyWithdraw(uint256 _pid) external whenNotPaused validPool(_pid) {
        Pool storage pool = pools[_pid];
        User storage user = users[_pid][msg.sender];
        
        uint256 amount = user.stAmount;
        require(amount > 0, "No staked amount");
        
        user.stAmount = 0;
        user.finishedMetaNode = 0;
        user.pendingMetaNode = 0;
        delete user.requests;
        
        pool.stTokenAmount -= amount;
        
        if (pool.stTokenAddress == address(0)) {
            payable(msg.sender).transfer(amount);
        } else {
            IERC20(pool.stTokenAddress).safeTransfer(msg.sender, amount);
        }
        
        emit Withdraw(msg.sender, _pid, amount);
    }

    // ============ 管理员函数 ============

    /**
     * @notice 设置每区块奖励数量
     * @dev 只有ADMIN_ROLE可以调用，修改前会更新所有池
     * @param _metaNodePerBlock 新的每区块奖励数量
     */
    function setMetaNodePerBlock(uint256 _metaNodePerBlock) external onlyRole(ADMIN_ROLE) {
        massUpdatePools();
        metaNodePerBlock = _metaNodePerBlock;
    }

    /**
     * @notice 设置池的激活状态
     * @dev 只有ADMIN_ROLE可以调用
     * @param _pid 池ID
     * @param _isActive 激活状态
     */
    function setPoolActive(uint256 _pid, bool _isActive) external onlyRole(ADMIN_ROLE) {
        require(_pid < pools.length, "Invalid pool ID");
        pools[_pid].isActive = _isActive;
    }

    // ============ 暂停控制函数 ============

    /**
     * @notice 设置质押操作的暂停状态
     * @dev 只有OPERATOR_ROLE可以调用
     * @param _paused 是否暂停
     */
    function setStakePaused(bool _paused) external onlyRole(OPERATOR_ROLE) {
        stakePaused = _paused;
    }

    /**
     * @notice 设置解质押操作的暂停状态
     * @dev 只有OPERATOR_ROLE可以调用
     * @param _paused 是否暂停
     */
    function setUnstakePaused(bool _paused) external onlyRole(OPERATOR_ROLE) {
        unstakePaused = _paused;
    }

    /**
     * @notice 设置领取奖励操作的暂停状态
     * @dev 只有OPERATOR_ROLE可以调用
     * @param _paused 是否暂停
     */
    function setClaimPaused(bool _paused) external onlyRole(OPERATOR_ROLE) {
        claimPaused = _paused;
    }

    /**
     * @notice 暂停所有操作
     * @dev 只有OPERATOR_ROLE可以调用
     */
    function pause() external onlyRole(OPERATOR_ROLE) {
        _pause();
    }

    /**
     * @notice 恢复所有操作
     * @dev 只有OPERATOR_ROLE可以调用
     */
    function unpause() external onlyRole(OPERATOR_ROLE) {
        _unpause();
    }

    // ============ 查询函数 ============

    /**
     * @notice 获取池的数量
     * @return 池的总数
     */
    function getPoolLength() external view returns (uint256) {
        return pools.length;
    }

    /**
     * @notice 获取用户在指定池的信息
     * @param _pid 池ID
     * @param _user 用户地址
     * @return stAmount 质押数量
     * @return pendingReward 待领取奖励
     * @return requests 解质押请求列表
     */
    function getUserInfo(uint256 _pid, address _user) external view returns (
        uint256 stAmount,
        uint256 pendingReward,
        UnstakeRequest[] memory requests
    ) {
        User memory user = users[_pid][_user];
        stAmount = user.stAmount;
        pendingReward = this.pendingMetaNode(_pid, _user);
        requests = user.requests;
    }

    /**
     * @notice 获取用户可提取的数量
     * @dev 计算已过锁定期的所有解质押请求总和
     * @param _pid 池ID
     * @param _user 用户地址
     * @return 可提取的数量
     */
    function getWithdrawableAmount(uint256 _pid, address _user) external view returns (uint256) {
        User memory user = users[_pid][_user];
        uint256 withdrawable = 0;
        
        for (uint256 i = 0; i < user.requests.length; i++) {
            if (user.requests[i].unlockBlock <= block.number) {
                withdrawable += user.requests[i].amount;
            }
        }
        
        return withdrawable;
    }

    // ============ 紧急函数 ============

    /**
     * @notice 紧急恢复意外发送到合约的代币
     * @dev 不能恢复奖励代币和质押池中的代币
     * @param _token 代币地址，address(0)表示ETH
     * @param _amount 恢复数量
     */
    function emergencyRecoverToken(address _token, uint256 _amount) external onlyRole(ADMIN_ROLE) {
        require(_token != address(metaNodeToken), "Cannot recover reward token");
        
        // Check if token is used in any pool
        for (uint256 i = 0; i < pools.length; i++) {
            require(pools[i].stTokenAddress != _token, "Cannot recover staked token");
        }
        
        if (_token == address(0)) {
            payable(msg.sender).transfer(_amount);
        } else {
            IERC20(_token).safeTransfer(msg.sender, _amount);
        }
    }

    /**
     * @notice 接收ETH
     * @dev 允许合约接收ETH用于ETH质押池
     */
    receive() external payable {}
}