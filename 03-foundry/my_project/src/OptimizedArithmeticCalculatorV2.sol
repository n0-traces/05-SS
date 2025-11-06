// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/**
 * @title SafeMath
 * @dev 安全数学运算库
 */
library SafeMath {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        unchecked {
            uint256 c = a + b;
            if (c < a) revert("SafeMath: addition overflow");
            return c;
        }
    }

    function sub(uint256 a, uint256 b) internal pure returns (uint256) {
        if (a < b) revert("SafeMath: subtraction underflow");
        return a - b;
    }

    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        if (a == 0) return 0;
        uint256 c = a * b;
        if (c / a != b) revert("SafeMath: multiplication overflow");
        return c;
    }

    function div(uint256 a, uint256 b) internal pure returns (uint256) {
        if (b == 0) revert("SafeMath: division by zero");
        return a / b;
    }
}

/**
 * @title OptimizedArithmeticCalculatorV2
 * @dev 优化策略2: 使用库函数、位操作优化、最小化存储
 * @author Foundry学习项目
 */
contract OptimizedArithmeticCalculatorV2 {
    using SafeMath for uint256;

    // 将多个变量打包到一个存储槽中
    uint256 private _packedData;
    // layout: [0-127: result][128-191: lastOperation][192-255: isInitialized]

    address private _owner;

    modifier onlyOwner() {
        require(msg.sender == _owner, "Only owner");
        _;
    }

    constructor() {
        _owner = msg.sender;
        _packedData = 1 << 255; // 初始化标志位
    }

    function _packData(uint256 result, uint8 operation) private {
        _packedData = (result & ((1 << 128) - 1)) |
                     (uint256(operation) << 128) |
                     (1 << 255);
    }

    function _unpackResult() private view returns (uint256) {
        return _packedData & ((1 << 128) - 1);
    }

    function _unpackOperation() private view returns (uint8) {
        return uint8((_packedData >> 128) & 0xFF);
    }

    /**
     * @dev 使用库函数的加法
     */
    function add(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        uint256 result = a.add(b);
        _packData(result, 0);
        return result;
    }

    /**
     * @dev 使用库函数的减法
     */
    function subtract(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        uint256 result = a.sub(b);
        _packData(result, 1);
        return result;
    }

    /**
     * @dev 使用库函数的乘法
     */
    function multiply(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        uint256 result = a.mul(b);
        _packData(result, 2);
        return result;
    }

    /**
     * @dev 使用库函数的除法
     */
    function divide(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        uint256 result = a.div(b);
        _packData(result, 3);
        return result;
    }

    /**
     * @dev 计算平方 - 优化常见用例
     */
    function square(uint256 a) external onlyOwner returns (uint256) {
        uint256 result = a.mul(a);
        _packData(result, 4);
        return result;
    }

    /**
     * @dev 计算立方
     */
    function cube(uint256 a) external onlyOwner returns (uint256) {
        uint256 result = a.mul(a).mul(a);
        _packData(result, 5);
        return result;
    }

    function getResult() external view returns (uint256) {
        return _unpackResult();
    }

    function getLastOperation() external view returns (uint8) {
        return _unpackOperation();
    }

    function isInitialized() external view returns (bool) {
        return (_packedData >> 255) == 1;
    }

    function reset() external onlyOwner {
        _packData(0, 0);
    }
}