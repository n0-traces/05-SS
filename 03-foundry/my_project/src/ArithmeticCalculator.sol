// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/**
 * @title ArithmeticCalculator
 * @dev 基本算术运算合约，用于演示 Gas 优化
 * @author Foundry学习项目
 */
contract ArithmeticCalculator {
    uint256 public result;
    address public owner;

    event CalculationPerformed(string operation, uint256 a, uint256 b, uint256 result);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can perform calculations");
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    /**
     * @dev 加法运算
     */
    function add(uint256 a, uint256 b) public onlyOwner returns (uint256) {
        result = a + b;
        emit CalculationPerformed("add", a, b, result);
        return result;
    }

    /**
     * @dev 减法运算
     */
    function subtract(uint256 a, uint256 b) public onlyOwner returns (uint256) {
        require(a >= b, "Subtraction underflow");
        result = a - b;
        emit CalculationPerformed("subtract", a, b, result);
        return result;
    }

    /**
     * @dev 乘法运算
     */
    function multiply(uint256 a, uint256 b) public onlyOwner returns (uint256) {
        result = a * b;
        emit CalculationPerformed("multiply", a, b, result);
        return result;
    }

    /**
     * @dev 除法运算
     */
    function divide(uint256 a, uint256 b) public onlyOwner returns (uint256) {
        require(b > 0, "Division by zero");
        result = a / b;
        emit CalculationPerformed("divide", a, b, result);
        return result;
    }

    /**
     * @dev 获取当前结果
     */
    function getResult() public view returns (uint256) {
        return result;
    }

    /**
     * @dev 重置结果
     */
    function reset() public onlyOwner {
        result = 0;
    }
}