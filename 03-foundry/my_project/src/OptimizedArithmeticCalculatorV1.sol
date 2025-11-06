// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/**
 * @title OptimizedArithmeticCalculatorV1
 * @dev 优化策略1: 移除事件，使用内置数学库，减少存储操作
 * @author Foundry学习项目
 */
contract OptimizedArithmeticCalculatorV1 {
    uint256 private _result;
    address private _owner;

    // 使用 packed 结构体来减少存储槽
    struct CalculationData {
        uint128 lastOperandA;
        uint128 lastOperandB;
    }

    CalculationData private _lastCalculation;

    modifier onlyOwner() {
        require(msg.sender == _owner, "Only owner");
        _;
    }

    constructor() {
        _owner = msg.sender;
    }

    /**
     * @dev 优化的加法 - 内联计算，减少存储操作
     */
    function add(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        assembly {
            if lt(add(a, b), a) { revert(0, 0) } // 检查溢出
        }
        _result = a + b;
        _lastCalculation = CalculationData(uint128(a), uint128(b));
        return _result;
    }

    /**
     * @dev 优化的减法 - 使用 require 内联检查
     */
    function subtract(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        require(a >= b, "Underflow");
        _result = a - b;
        _lastCalculation = CalculationData(uint128(a), uint128(b));
        return _result;
    }

    /**
     * @dev 优化的乘法 - 溢出检查
     */
    function multiply(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        if (a == 0) return 0;
        require(b == 0 || a <= type(uint256).max / b, "Overflow");
        _result = a * b;
        _lastCalculation = CalculationData(uint128(a), uint128(b));
        return _result;
    }

    /**
     * @dev 优化的除法
     */
    function divide(uint256 a, uint256 b) external onlyOwner returns (uint256) {
        require(b > 0, "Division by zero");
        _result = a / b;
        _lastCalculation = CalculationData(uint128(a), uint128(b));
        return _result;
    }

    /**
     * @dev 批量操作 - 减少函数调用开销
     */
    function batchOperation(
        uint256 a,
        uint256 b,
        uint8 operation
    ) external onlyOwner returns (uint256) {
        if (operation == 0) return this.add(a, b);
        if (operation == 1) return this.subtract(a, b);
        if (operation == 2) return this.multiply(a, b);
        if (operation == 3) return this.divide(a, b);
        revert("Invalid operation");
    }

    function getResult() external view returns (uint256) {
        return _result;
    }

    function getLastCalculation() external view returns (uint128 a, uint128 b) {
        a = _lastCalculation.lastOperandA;
        b = _lastCalculation.lastOperandB;
    }

    function reset() external onlyOwner {
        _result = 0;
        delete _lastCalculation;
    }
}