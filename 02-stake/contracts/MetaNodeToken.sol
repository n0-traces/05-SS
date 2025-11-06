// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title MetaNodeToken
 * @author MetaNode Academy
 * @notice MetaNode质押系统的奖励代币
 * @dev 基于ERC20标准的代币合约，具有固定供应上限
 *
 * 特点:
 * - 总供应量上限: 10亿枚
 * - 只有owner可以铸造新代币
 * - 铸造不能超过最大供应量
 */
contract MetaNodeToken is ERC20, Ownable {
    /// @notice 最大供应量: 10亿枚代币
    uint256 public constant MAX_SUPPLY = 1000000000 * 10**18;

    /**
     * @notice 构造函数
     * @dev 部署时铸造所有代币给部署者
     */
    constructor() ERC20("MetaNode Token", "META") Ownable(msg.sender) {
        _mint(msg.sender, MAX_SUPPLY);
    }

    /**
     * @notice 铸造新代币
     * @dev 只有owner可以调用，不能超过最大供应量
     * @param to 接收地址
     * @param amount 铸造数量
     */
    function mint(address to, uint256 amount) external onlyOwner {
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(to, amount);
    }
}
