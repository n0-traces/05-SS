// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {OptimizedArithmeticCalculatorV2} from "../src/OptimizedArithmeticCalculatorV2.sol";
import "forge-std/console.sol";

contract OptimizedArithmeticCalculatorV2Test is Test {
    OptimizedArithmeticCalculatorV2 public calculator;

    // 记录V2版本的 Gas 消耗
    uint256 public addGasUsedV2;
    uint256 public subtractGasUsedV2;
    uint256 public multiplyGasUsedV2;
    uint256 public divideGasUsedV2;
    uint256 public resetGasUsedV2;
    uint256 public squareGasUsedV2;
    uint256 public cubeGasUsedV2;

    function setUp() public {
        calculator = new OptimizedArithmeticCalculatorV2();
    }

    function test_AddV2() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.add(10, 20);
        uint256 gasEnd = gasleft();
        addGasUsedV2 = gasStart - gasEnd;

        assertEq(result, 30, "Addition result incorrect");
        assertEq(calculator.getLastOperation(), 0, "Last operation code incorrect");
        assertEq(calculator.getResult(), 30, "Stored result incorrect");

        console.log("V2 Add operation gas used:", addGasUsedV2);
    }

    function test_SubtractV2() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.subtract(50, 20);
        uint256 gasEnd = gasleft();
        subtractGasUsedV2 = gasStart - gasEnd;

        assertEq(result, 30, "Subtraction result incorrect");
        console.log("V2 Subtract operation gas used:", subtractGasUsedV2);
    }

    function test_MultiplyV2() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.multiply(5, 6);
        uint256 gasEnd = gasleft();
        multiplyGasUsedV2 = gasStart - gasEnd;

        assertEq(result, 30, "Multiplication result incorrect");
        console.log("V2 Multiply operation gas used:", multiplyGasUsedV2);
    }

    function test_DivideV2() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.divide(60, 2);
        uint256 gasEnd = gasleft();
        divideGasUsedV2 = gasStart - gasEnd;

        assertEq(result, 30, "Division result incorrect");
        console.log("V2 Divide operation gas used:", divideGasUsedV2);
    }

    function test_SquareV2() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.square(5);
        uint256 gasEnd = gasleft();
        squareGasUsedV2 = gasStart - gasEnd;

        assertEq(result, 25, "Square result incorrect");
        console.log("V2 Square operation gas used:", squareGasUsedV2);
    }

    function test_CubeV2() public {
        uint256 gasStart = gasleft();
        uint256 result = calculator.cube(3);
        uint256 gasEnd = gasleft();
        cubeGasUsedV2 = gasStart - gasEnd;

        assertEq(result, 27, "Cube result incorrect");
        console.log("V2 Cube operation gas used:", cubeGasUsedV2);
    }

    function test_ResetV2() public {
        calculator.add(10, 20); // Set a value first

        uint256 gasStart = gasleft();
        calculator.reset();
        uint256 gasEnd = gasleft();
        resetGasUsedV2 = gasStart - gasEnd;

        assertEq(calculator.getResult(), 0, "Reset result incorrect");
        console.log("V2 Reset operation gas used:", resetGasUsedV2);
    }

    function test_GasSummaryV2() public {
        // 执行所有操作并记录 Gas 消耗
        calculator.add(10, 20);
        calculator.subtract(30, 10);
        calculator.multiply(5, 4);
        calculator.divide(40, 2);
        calculator.square(6);
        calculator.cube(3);
        calculator.reset();

        console.log("=== V2 Gas Consumption Summary ===");
        console.log("V2 Add gas used:", addGasUsedV2);
        console.log("V2 Subtract gas used:", subtractGasUsedV2);
        console.log("V2 Multiply gas used:", multiplyGasUsedV2);
        console.log("V2 Divide gas used:", divideGasUsedV2);
        console.log("V2 Square gas used:", squareGasUsedV2);
        console.log("V2 Cube gas used:", cubeGasUsedV2);
        console.log("V2 Reset gas used:", resetGasUsedV2);
    }
}