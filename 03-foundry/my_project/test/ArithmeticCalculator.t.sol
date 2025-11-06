// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ArithmeticCalculator} from "../src/ArithmeticCalculator.sol";
import "forge-std/console.sol";

contract ArithmeticCalculatorTest is Test {
    ArithmeticCalculator public calculator;

    // 记录 Gas 消耗的变量
    uint256 public addGasUsed;
    uint256 public subtractGasUsed;
    uint256 public multiplyGasUsed;
    uint256 public divideGasUsed;
    uint256 public resetGasUsed;

    function setUp() public {
        calculator = new ArithmeticCalculator();
    }

    function test_Add() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.add(10, 20);
        uint256 gasEnd = gasleft();
        addGasUsed = gasStart - gasEnd;

        assertEq(result, 30, "Addition result incorrect");
        assertEq(calculator.getResult(), 30, "Stored result incorrect");

        console.log("Add operation gas used:", addGasUsed);
    }

    function test_Subtract() public {
        calculator.add(50, 20); // Set initial value

        uint256 gasStart = gasleft();
        uint256 result = calculator.subtract(50, 20);
        uint256 gasEnd = gasleft();
        subtractGasUsed = gasStart - gasEnd;

        assertEq(result, 30, "Subtraction result incorrect");
        assertEq(calculator.getResult(), 30, "Stored result incorrect");

        console.log("Subtract operation gas used:", subtractGasUsed);
    }

    function test_Multiply() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.multiply(5, 6);
        uint256 gasEnd = gasleft();
        multiplyGasUsed = gasStart - gasEnd;

        assertEq(result, 30, "Multiplication result incorrect");
        assertEq(calculator.getResult(), 30, "Stored result incorrect");

        console.log("Multiply operation gas used:", multiplyGasUsed);
    }

    function test_Divide() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.divide(60, 2);
        uint256 gasEnd = gasleft();
        divideGasUsed = gasStart - gasEnd;

        assertEq(result, 30, "Division result incorrect");
        assertEq(calculator.getResult(), 30, "Stored result incorrect");

        console.log("Divide operation gas used:", divideGasUsed);
    }

    function test_Reset() public {
        calculator.add(10, 20); // Set a value first

        uint256 gasStart = gasleft();
        calculator.reset();
        uint256 gasEnd = gasleft();
        resetGasUsed = gasStart - gasEnd;

        assertEq(calculator.getResult(), 0, "Reset result incorrect");

        console.log("Reset operation gas used:", resetGasUsed);
    }

    // 模糊测试 - 验证加法结合律
    function testFuzz_AddAssociative(uint256 a, uint256 b, uint256 c) public {
        // 防止溢出
        vm.assume(a <= type(uint256).max - b - c);

        calculator.add(a, b);
        uint256 result1 = calculator.add(calculator.getResult(), c);

        calculator.reset();
        calculator.add(b, c);
        uint256 result2 = calculator.add(a, calculator.getResult());

        assertEq(result1, result2, "Addition associative property failed");
    }

    // 边界条件测试
    function test_SubtractUnderflow() public {
        vm.expectRevert("Subtraction underflow");
        calculator.subtract(10, 20);
    }

    function test_DivideByZero() public {
        vm.expectRevert("Division by zero");
        calculator.divide(10, 0);
    }

    // Gas 性能汇总测试
    function test_GasSummary() public {
        // 执行所有操作并记录 Gas 消耗
        calculator.add(10, 20);
        calculator.subtract(30, 10);
        calculator.multiply(5, 4);
        calculator.divide(40, 2);
        calculator.reset();

        console.log("=== Gas Consumption Summary ===");
        console.log("Add gas used:", addGasUsed);
        console.log("Subtract gas used:", subtractGasUsed);
        console.log("Multiply gas used:", multiplyGasUsed);
        console.log("Divide gas used:", divideGasUsed);
        console.log("Reset gas used:", resetGasUsed);
        console.log("Total gas for all operations:",
                    addGasUsed + subtractGasUsed + multiplyGasUsed + divideGasUsed + resetGasUsed);
    }
}