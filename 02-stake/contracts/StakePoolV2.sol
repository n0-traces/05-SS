// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./StakePool.sol";

/**
 * @title StakePoolV2
 * @author MetaNode Academy
 * @notice StakePool的升级版本，增加了奖励倍数功能
 * @dev 继承StakePool，添加奖励倍数机制
 *
 * 新增功能:
 * - 可配置的奖励倍数
 * - 记录用户总领取奖励
 * - 带倍数的奖励领取
 */
contract StakePoolV2 is StakePool {
    /// @notice 用户总领取奖励映射
    mapping(address => uint256) public userTotalRewardsClaimed;

    /// @notice 奖励倍数 (100 = 1x, 200 = 2x)
    uint256 public bonusMultiplier = 100;

    /// @notice 奖励倍数更新事件
    /// @param newMultiplier 新的奖励倍数
    event BonusMultiplierUpdated(uint256 newMultiplier);

    /// @notice 用户领取奖励事件
    /// @param user 用户地址
    /// @param amount 领取数量
    event UserTotalRewardsClaimed(address indexed user, uint256 amount);

    /**
     * @notice 设置奖励倍数
     * @dev 只有ADMIN_ROLE可以调用，倍数不能小于100
     * @param _multiplier 新的奖励倍数 (100 = 1x, 200 = 2x)
     */
    function setBonusMultiplier(uint256 _multiplier) external onlyRole(ADMIN_ROLE) {
        require(_multiplier >= 100, "Multiplier must be >= 100");
        bonusMultiplier = _multiplier;
        emit BonusMultiplierUpdated(_multiplier);
    }

    /**
     * @notice 查询带倍数的待领取奖励
     * @dev 基础奖励乘以奖励倍数
     * @param _pid 池ID
     * @param _user 用户地址
     * @return 带倍数的待领取奖励
     */
    function pendingMetaNodeWithBonus(uint256 _pid, address _user) external view validPool(_pid) returns (uint256) {
        uint256 basePending = this.pendingMetaNode(_pid, _user);
        return (basePending * bonusMultiplier) / 100;
    }

    /**
     * @notice 领取带倍数的奖励
     * @dev 领取时应用奖励倍数，并记录到用户总领取数
     * @param _pid 池ID
     */
    function claimWithBonus(uint256 _pid) external whenClaimNotPaused whenNotPaused validPool(_pid) {
        Pool storage pool = pools[_pid];
        User storage user = users[_pid][msg.sender];

        updatePoolReward(_pid);

        uint256 pending = ((user.stAmount * pool.accMetaNodePerST) / 1e12) - user.finishedMetaNode;
        uint256 totalPending = pending + user.pendingMetaNode;

        require(totalPending > 0, "No pending rewards");

        user.finishedMetaNode = (user.stAmount * pool.accMetaNodePerST) / 1e12;
        user.pendingMetaNode = 0;

        uint256 bonusAmount = (totalPending * bonusMultiplier) / 100;
        userTotalRewardsClaimed[msg.sender] += bonusAmount;

        metaNodeToken.transfer(msg.sender, bonusAmount);

        emit Claim(msg.sender, _pid, bonusAmount);
        emit UserTotalRewardsClaimed(msg.sender, bonusAmount);
    }

    /**
     * @notice 获取合约版本
     * @return 版本号字符串
     */
    function version() external pure returns (string memory) {
        return "2.0.0";
    }
}
