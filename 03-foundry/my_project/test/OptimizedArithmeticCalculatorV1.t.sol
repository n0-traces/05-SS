// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {OptimizedArithmeticCalculatorV1} from "../src/OptimizedArithmeticCalculatorV1.sol";
import "forge-std/console.sol";

contract OptimizedArithmeticCalculatorV1Test is Test {
    OptimizedArithmeticCalculatorV1 public calculator;

    // 记录优化版本的 Gas 消耗
    uint256 public addGasUsedV1;
    uint256 public subtractGasUsedV1;
    uint256 public multiplyGasUsedV1;
    uint256 public divideGasUsedV1;
    uint256 public resetGasUsedV1;
    uint256 public batchOperationGasUsedV1;

    function setUp() public {
        calculator = new OptimizedArithmeticCalculatorV1();
    }

    function test_AddV1() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.add(10, 20);
        uint256 gasEnd = gasleft();
        addGasUsedV1 = gasStart - gasEnd;

        assertEq(result, 30, "Addition result incorrect");
        (uint128 a, uint128 b) = calculator.getLastCalculation();
        assertEq(a, 10, "Last operand A incorrect");
        assertEq(b, 20, "Last operand B incorrect");

        console.log("V1 Add operation gas used:", addGasUsedV1);
    }

    function test_SubtractV1() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.subtract(50, 20);
        uint256 gasEnd = gasleft();
        subtractGasUsedV1 = gasStart - gasEnd;

        assertEq(result, 30, "Subtraction result incorrect");
        console.log("V1 Subtract operation gas used:", subtractGasUsedV1);
    }

    function test_MultiplyV1() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.multiply(5, 6);
        uint256 gasEnd = gasleft();
        multiplyGasUsedV1 = gasStart - gasEnd;

        assertEq(result, 30, "Multiplication result incorrect");
        console.log("V1 Multiply operation gas used:", multiplyGasUsedV1);
    }

    function test_DivideV1() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.divide(60, 2);
        uint256 gasEnd = gasleft();
        divideGasUsedV1 = gasStart - gasEnd;

        assertEq(result, 30, "Division result incorrect");
        console.log("V1 Divide operation gas used:", divideGasUsedV1);
    }

    function test_ResetV1() public {
        calculator.add(10, 20); // Set a value first

        uint256 gasStart = gasleft();
        calculator.reset();
        uint256 gasEnd = gasleft();
        resetGasUsedV1 = gasStart - gasEnd;

        assertEq(calculator.getResult(), 0, "Reset result incorrect");
        console.log("V1 Reset operation gas used:", resetGasUsedV1);
    }

    function test_BatchOperationV1() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.batchOperation(10, 20, 0); // addition
        uint256 gasEnd = gasleft();
        batchOperationGasUsedV1 = gasStart - gasEnd;

        assertEq(result, 30, "Batch operation result incorrect");
        console.log("V1 Batch operation gas used:", batchOperationGasUsedV1);
    }

    function test_GasSummaryV1() public {
        // 执行所有操作并记录 Gas 消耗
        calculator.add(10, 20);
        calculator.subtract(30, 10);
        calculator.multiply(5, 4);
        calculator.divide(40, 2);
        calculator.reset();

        console.log("=== V1 Gas Consumption Summary ===");
        console.log("V1 Add gas used:", addGasUsedV1);
        console.log("V1 Subtract gas used:", subtractGasUsedV1);
        console.log("V1 Multiply gas used:", multiplyGasUsedV1);
        console.log("V1 Divide gas used:", divideGasUsedV1);
        console.log("V1 Reset gas used:", resetGasUsedV1);
        console.log("V1 Batch operation gas used:", batchOperationGasUsedV1);
    }
}