// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/**
 * @title TestToken
 * @author MetaNode Academy
 * @notice 用于测试的ERC20代币合约
 * @dev 支持自定义小数位数和公开铸造
 *
 * 特点:
 * - 可自定义代币名称、符号、精度
 * - 任何人都可以铸造代币(用于测试)
 * - 部署时为部署者铸造初始供应量
 */
contract TestToken is ERC20 {
    /// @dev 代币精度
    uint8 private _decimals;

    /**
     * @notice 构造函数
     * @param name 代币名称
     * @param symbol 代币符号
     * @param decimals_ 代币精度
     * @param totalSupply_ 初始供应量(不含精度)
     */
    constructor(
        string memory name,
        string memory symbol,
        uint8 decimals_,
        uint256 totalSupply_
    ) ERC20(name, symbol) {
        _decimals = decimals_;
        _mint(msg.sender, totalSupply_ * 10**decimals_);
    }

    /**
     * @notice 获取代币精度
     * @return 代币小数位数
     */
    function decimals() public view virtual override returns (uint8) {
        return _decimals;
    }

    /**
     * @notice 铸造代币
     * @dev 公开函数，任何人都可以调用(仅用于测试)
     * @param to 接收地址
     * @param amount 铸造数量
     */
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
