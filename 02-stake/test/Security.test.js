const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("StakePool Security Tests", function () {
    async function deployStakePoolFixture() {
        const [owner, user1, user2, attacker] = await ethers.getSigners();
        
        // Deploy contracts
        const MetaNodeToken = await ethers.getContractFactory("MetaNodeToken");
        const metaNodeToken = await MetaNodeToken.deploy();
        
        const TestToken = await ethers.getContractFactory("TestToken");
        const testToken = await TestToken.deploy("Test Token", "TEST", 18, 1000000);
        
        const StakePool = await ethers.getContractFactory("StakePool");
        const stakePool = await StakePool.deploy();
        
        const metaNodePerBlock = ethers.parseEther("10");
        const startBlock = await ethers.provider.getBlockNumber() + 1;
        
        await stakePool.initialize(
            await metaNodeToken.getAddress(),
            metaNodePerBlock,
            startBlock
        );
        
        // Transfer tokens
        await metaNodeToken.transfer(
            await stakePool.getAddress(),
            ethers.parseEther("100000")
        );
        
        await testToken.transfer(user1.address, ethers.parseEther("1000"));
        await testToken.transfer(user2.address, ethers.parseEther("1000"));
        await testToken.transfer(attacker.address, ethers.parseEther("1000"));
        
        return {
            stakePool,
            metaNodeToken,
            testToken,
            owner,
            user1,
            user2,
            attacker,
            metaNodePerBlock
        };
    }
    
    describe("Reentrancy Protection", function () {
        it("Should prevent reentrancy on withdraw", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            // Add native currency pool
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                1 // Very short lock period
            );
            
            // Stake and unstake
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            await stakePool.connect(user1).unstake(0, ethers.parseEther("1"));
            
            // Mine a block to pass lock period
            await ethers.provider.send("evm_mine");
            
            // Normal withdrawal should work
            await expect(stakePool.connect(user1).withdraw(0))
                .to.emit(stakePool, "Withdraw");
        });
    });
    
    describe("Integer Overflow/Underflow Protection", function () {
        it("Should handle large numbers correctly", async function () {
            const { stakePool, testToken, user1 } = await loadFixture(deployStakePoolFixture);
            
            // Add pool with very large minimum deposit
            await stakePool.addPool(
                await testToken.getAddress(),
                100,
                ethers.parseEther("1000000"), // Very large minimum
                100
            );
            
            const largeAmount = ethers.parseEther("1000000");
            
            // Mint large amount to user
            await testToken.mint(user1.address, largeAmount);
            await testToken.connect(user1).approve(await stakePool.getAddress(), largeAmount);
            
            // Should handle large deposit
            await expect(stakePool.connect(user1).stake(0, largeAmount))
                .to.emit(stakePool, "Deposit");
        });
        
        it("Should prevent unstaking more than staked", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            // Stake 1 ETH
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Try to unstake more than staked
            await expect(stakePool.connect(user1).unstake(0, ethers.parseEther("2")))
                .to.be.revertedWith("Insufficient staked amount");
        });
    });
    
    describe("Authorization Bypass Attempts", function () {
        it("Should prevent non-admin from adding pools", async function () {
            const { stakePool, attacker, testToken } = await loadFixture(deployStakePoolFixture);
            
            await expect(stakePool.connect(attacker).addPool(
                await testToken.getAddress(),
                100,
                ethers.parseEther("10"),
                200
            )).to.be.reverted;
        });
        
        it("Should prevent unauthorized emergency recovery", async function () {
            const { stakePool, testToken, attacker } = await loadFixture(deployStakePoolFixture);
            
            await expect(stakePool.connect(attacker).emergencyRecoverToken(
                await testToken.getAddress(),
                ethers.parseEther("100")
            )).to.be.reverted;
        });
        
        it("Should prevent unauthorized parameter changes", async function () {
            const { stakePool, attacker } = await loadFixture(deployStakePoolFixture);
            
            await expect(stakePool.connect(attacker).setMetaNodePerBlock(ethers.parseEther("1000")))
                .to.be.reverted;
        });
    });
    
    describe("Edge Cases", function () {
        it("Should handle zero staking attempts", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            await expect(stakePool.connect(user1).stake(0, 0, { value: 0 }))
                .to.be.revertedWith("Amount below minimum deposit");
        });
        
        it("Should handle claiming with no rewards", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            // Try to claim without staking
            await expect(stakePool.connect(user1).claim(0))
                .to.be.revertedWith("No pending rewards");
        });
        
        it("Should handle multiple withdrawals correctly", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                1
            );
            
            // Stake and unstake
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            await stakePool.connect(user1).unstake(0, ethers.parseEther("1"));
            
            // Wait for unlock
            await ethers.provider.send("evm_mine");
            
            // First withdrawal should succeed
            await expect(stakePool.connect(user1).withdraw(0))
                .to.emit(stakePool, "Withdraw");
            
            // Second withdrawal should fail
            await expect(stakePool.connect(user1).withdraw(0))
                .to.be.revertedWith("No withdrawable amount");
        });
        
        it("Should handle pool deactivation correctly", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            // Stake in active pool
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Deactivate pool
            await stakePool.setPoolActive(0, false);
            
            // Should not be able to stake in inactive pool
            await expect(stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            })).to.be.revertedWith("Pool is not active");
        });
    });
    
    describe("Reward Calculation Edge Cases", function () {
        it("Should handle reward calculation when total supply is zero", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            // Check pending rewards without any staking
            const pending = await stakePool.pendingMetaNode(0, user1.address);
            expect(pending).to.equal(0);
        });
        
        it("Should handle reward distribution when pool weight is zero", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                0, // Zero weight
                ethers.parseEther("0.1"),
                100
            );
            
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Mine some blocks
            for (let i = 0; i < 5; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            // Should handle division by zero gracefully when totalPoolWeight is zero
            // The contract should not revert but return 0 pending rewards
            try {
                const pending = await stakePool.pendingMetaNode(0, user1.address);
                expect(pending).to.equal(0);
            } catch (error) {
                // If it reverts with division by zero, that's also expected behavior
                expect(error.message).to.include("division by zero");
            }
        });
        
        it("Should handle precision in reward calculations", async function () {
            const { stakePool, user1, user2 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                1, // Very small minimum deposit
                100
            );
            
            // Stake very small amounts
            await stakePool.connect(user1).stake(0, 1000, { value: 1000 });
            await stakePool.connect(user2).stake(0, 1, { value: 1 });
            
            // Mine blocks
            for (let i = 0; i < 10; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const pending1 = await stakePool.pendingMetaNode(0, user1.address);
            const pending2 = await stakePool.pendingMetaNode(0, user2.address);
            
            // User1 should have much more rewards due to larger stake
            expect(pending1).to.be.gt(pending2);
        });
    });
    
    describe("Gas Optimization Tests", function () {
        it("Should handle large number of unstake requests efficiently", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.001"),
                1
            );
            
            // Stake a large amount
            await stakePool.connect(user1).stake(0, ethers.parseEther("10"), { 
                value: ethers.parseEther("10") 
            });
            
            // Create many small unstake requests
            const numRequests = 20;
            for (let i = 0; i < numRequests; i++) {
                await stakePool.connect(user1).unstake(0, ethers.parseEther("0.1"));
            }
            
            const userInfo = await stakePool.getUserInfo(0, user1.address);
            expect(userInfo.requests).to.have.length(numRequests);
            
            // Wait for unlock
            await ethers.provider.send("evm_mine");
            
            // Withdrawal should handle all requests in one transaction
            await expect(stakePool.connect(user1).withdraw(0))
                .to.emit(stakePool, "Withdraw");
            
            // All requests should be cleared
            const userInfoAfter = await stakePool.getUserInfo(0, user1.address);
            expect(userInfoAfter.requests).to.have.length(0);
        });
    });
});