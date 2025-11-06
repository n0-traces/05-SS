const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("Enhanced StakePool Tests - Coverage Improvement", function () {
    async function deployStakePoolFixture() {
        const [owner, user1, user2, user3] = await ethers.getSigners();
        
        // Deploy contracts
        const MetaNodeToken = await ethers.getContractFactory("MetaNodeToken");
        const metaNodeToken = await MetaNodeToken.deploy();
        
        const TestToken = await ethers.getContractFactory("TestToken");
        const testToken = await TestToken.deploy("Test Token", "TEST", 18, 1000000);
        
        const StakePool = await ethers.getContractFactory("StakePool");
        const stakePool = await StakePool.deploy();
        
        const StakePoolV2 = await ethers.getContractFactory("StakePoolV2");
        const stakePoolV2 = await StakePoolV2.deploy();
        
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
        await testToken.transfer(user3.address, ethers.parseEther("1000"));
        
        return {
            stakePool,
            stakePoolV2,
            metaNodeToken,
            testToken,
            owner,
            user1,
            user2,
            user3,
            metaNodePerBlock,
            startBlock
        };
    }
    
    describe("MetaNodeToken Coverage", function () {
        it("Should mint tokens by owner", async function () {
            const { owner, user1 } = await loadFixture(deployStakePoolFixture);
            
            // Deploy a fresh token for this test to avoid max supply issues
            const MetaNodeToken = await ethers.getContractFactory("MetaNodeToken");
            const freshToken = await MetaNodeToken.deploy();
            
            const mintAmount = ethers.parseEther("1000");
            await expect(freshToken.mint(user1.address, mintAmount))
                .to.emit(freshToken, "Transfer")
                .withArgs(ethers.ZeroAddress, user1.address, mintAmount);
                
            expect(await freshToken.balanceOf(user1.address)).to.equal(mintAmount);
        });
        
        it("Should fail to mint beyond max supply", async function () {
            const { metaNodeToken, user1 } = await loadFixture(deployStakePoolFixture);
            
            const maxSupply = ethers.parseEther("1000000000");
            const excessAmount = ethers.parseEther("1");
            
            await expect(metaNodeToken.mint(user1.address, excessAmount))
                .to.be.revertedWith("Exceeds max supply");
        });
        
        it("Should not allow non-owner to mint", async function () {
            const { metaNodeToken, user1 } = await loadFixture(deployStakePoolFixture);
            
            await expect(metaNodeToken.connect(user1).mint(user1.address, ethers.parseEther("100")))
                .to.be.revertedWithCustomError(metaNodeToken, "OwnableUnauthorizedAccount");
        });
    });
    
    describe("TestToken Coverage", function () {
        it("Should return correct decimals", async function () {
            const { testToken } = await loadFixture(deployStakePoolFixture);
            expect(await testToken.decimals()).to.equal(18);
        });
        
        it("Should allow anyone to mint", async function () {
            const { testToken, user1 } = await loadFixture(deployStakePoolFixture);
            
            const mintAmount = ethers.parseEther("500");
            await expect(testToken.connect(user1).mint(user1.address, mintAmount))
                .to.emit(testToken, "Transfer")
                .withArgs(ethers.ZeroAddress, user1.address, mintAmount);
        });
        
        it("Should create token with custom decimals", async function () {
            const TestToken = await ethers.getContractFactory("TestToken");
            const customToken = await TestToken.deploy("Custom Token", "CUSTOM", 6, 1000000);
            
            expect(await customToken.decimals()).to.equal(6);
            expect(await customToken.totalSupply()).to.equal(BigInt(1000000) * BigInt(10**6));
        });
    });
    
    describe("StakePoolV2 Coverage", function () {
        async function setupV2Fixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            await fixture.stakePoolV2.initialize(
                await fixture.metaNodeToken.getAddress(),
                fixture.metaNodePerBlock,
                fixture.startBlock
            );
            
            // Transfer tokens to V2 contract
            await fixture.metaNodeToken.transfer(
                await fixture.stakePoolV2.getAddress(),
                ethers.parseEther("100000")
            );
            
            // Add a pool
            await fixture.stakePoolV2.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            return fixture;
        }
        
        it("Should set bonus multiplier", async function () {
            const { stakePoolV2 } = await loadFixture(setupV2Fixture);
            
            await expect(stakePoolV2.setBonusMultiplier(200))
                .to.emit(stakePoolV2, "BonusMultiplierUpdated")
                .withArgs(200);
                
            expect(await stakePoolV2.bonusMultiplier()).to.equal(200);
        });
        
        it("Should fail to set bonus multiplier below 100", async function () {
            const { stakePoolV2 } = await loadFixture(setupV2Fixture);
            
            await expect(stakePoolV2.setBonusMultiplier(50))
                .to.be.revertedWith("Multiplier must be >= 100");
        });
        
        it("Should calculate pending rewards with bonus", async function () {
            const { stakePoolV2, user1 } = await loadFixture(setupV2Fixture);
            
            // Set 2x bonus
            await stakePoolV2.setBonusMultiplier(200);
            
            // Stake some ETH
            await stakePoolV2.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Mine some blocks
            for (let i = 0; i < 5; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const basePending = await stakePoolV2.pendingMetaNode(0, user1.address);
            const bonusPending = await stakePoolV2.pendingMetaNodeWithBonus(0, user1.address);
            
            expect(bonusPending).to.equal(basePending * BigInt(2));
        });
        
        it("Should claim rewards with bonus", async function () {
            const { stakePoolV2, metaNodeToken, user1 } = await loadFixture(setupV2Fixture);
            
            // Set 1.5x bonus
            await stakePoolV2.setBonusMultiplier(150);
            
            // Stake some ETH
            await stakePoolV2.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Mine some blocks
            for (let i = 0; i < 10; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const tx = await stakePoolV2.connect(user1).claimWithBonus(0);
            const receipt = await tx.wait();
            
            // Find the UserTotalRewardsClaimed event to get actual amount
            const userTotalEvent = receipt.logs.find(log => {
                try {
                    const parsed = stakePoolV2.interface.parseLog(log);
                    return parsed.name === "UserTotalRewardsClaimed";
                } catch {
                    return false;
                }
            });
            
            expect(userTotalEvent).to.not.be.undefined;
            const parsedEvent = stakePoolV2.interface.parseLog(userTotalEvent);
            const actualAmount = parsedEvent.args[1];
                
            expect(await stakePoolV2.userTotalRewardsClaimed(user1.address)).to.equal(actualAmount);
        });
        
        it("Should return correct version", async function () {
            const { stakePoolV2 } = await loadFixture(setupV2Fixture);
            expect(await stakePoolV2.version()).to.equal("2.0.0");
        });
        
        it("Should not allow non-admin to set bonus multiplier", async function () {
            const { stakePoolV2, user1 } = await loadFixture(setupV2Fixture);
            
            await expect(stakePoolV2.connect(user1).setBonusMultiplier(200))
                .to.be.reverted;
        });
    });
    
    describe("Advanced StakePool Functions", function () {
        async function setupAdvancedFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            // Add multiple pools
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            await fixture.stakePool.addPool(
                await fixture.testToken.getAddress(),
                200,
                ethers.parseEther("10"),
                200
            );
            
            return fixture;
        }
        
        it("Should handle massUpdatePools function", async function () {
            const { stakePool, user1, testToken } = await loadFixture(setupAdvancedFixture);
            
            // Stake in both pools
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            await testToken.connect(user1).approve(await stakePool.getAddress(), ethers.parseEther("100"));
            await stakePool.connect(user1).stake(1, ethers.parseEther("100"));
            
            // Call massUpdatePools
            await expect(stakePool.massUpdatePools()).to.not.be.reverted;
        });
        
        it("Should handle getPoolLength function", async function () {
            const { stakePool } = await loadFixture(setupAdvancedFixture);
            expect(await stakePool.getPoolLength()).to.equal(2);
        });
        
        it("Should handle getWithdrawableAmount function", async function () {
            const { stakePool, user1 } = await loadFixture(setupAdvancedFixture);
            
            // Stake and unstake
            await stakePool.connect(user1).stake(0, ethers.parseEther("2"), { 
                value: ethers.parseEther("2") 
            });
            await stakePool.connect(user1).unstake(0, ethers.parseEther("1"));
            
            // Should be 0 initially (locked)
            expect(await stakePool.getWithdrawableAmount(0, user1.address)).to.equal(0);
            
            // Mine blocks to pass lock period
            for (let i = 0; i < 105; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            // Should be withdrawable now
            expect(await stakePool.getWithdrawableAmount(0, user1.address)).to.equal(ethers.parseEther("1"));
        });
        
        it("Should handle setMetaNodePerBlock function", async function () {
            const { stakePool } = await loadFixture(setupAdvancedFixture);
            
            const newRate = ethers.parseEther("20");
            await stakePool.setMetaNodePerBlock(newRate);
            expect(await stakePool.metaNodePerBlock()).to.equal(newRate);
        });
        
        it("Should handle receive function", async function () {
            const { stakePool, user1 } = await loadFixture(setupAdvancedFixture);
            
            // Send ETH directly to contract
            const tx = {
                to: await stakePool.getAddress(),
                value: ethers.parseEther("1")
            };
            
            await expect(user1.sendTransaction(tx)).to.not.be.reverted;
        });
        
        it("Should handle emergencyRecoverToken function", async function () {
            const { stakePool, testToken, metaNodeToken, owner } = await loadFixture(setupAdvancedFixture);
            
            // Create a different token for recovery
            const TestToken = await ethers.getContractFactory("TestToken");
            const randomToken = await TestToken.deploy("Random", "RND", 18, 1000000);
            
            // Send some random tokens to the contract
            await randomToken.transfer(await stakePool.getAddress(), ethers.parseEther("100"));
            
            // Should be able to recover random token
            await expect(stakePool.emergencyRecoverToken(
                await randomToken.getAddress(),
                ethers.parseEther("50")
            )).to.not.be.reverted;
            
            // Should not be able to recover MetaNode token
            await expect(stakePool.emergencyRecoverToken(
                await metaNodeToken.getAddress(),
                ethers.parseEther("50")
            )).to.be.revertedWith("Cannot recover reward token");
            
            // Should not be able to recover staked token
            await expect(stakePool.emergencyRecoverToken(
                await testToken.getAddress(),
                ethers.parseEther("50")
            )).to.be.revertedWith("Cannot recover staked token");
        });
        
        it("Should handle emergencyRecoverToken for ETH", async function () {
            const { stakePool, user1 } = await loadFixture(setupAdvancedFixture);
            
            // Send ETH to contract
            await user1.sendTransaction({
                to: await stakePool.getAddress(),
                value: ethers.parseEther("1")
            });
            
            // Should be able to recover ETH (since ETH is not a staked token in any active pool)
            // Note: ETH is used in pool 0, so this might fail. Let's check the pool status first
            const pool0 = await stakePool.pools(0);
            
            if (pool0.stTokenAddress === ethers.ZeroAddress) {
                // ETH is a staked token, so recovery should fail
                await expect(stakePool.emergencyRecoverToken(
                    ethers.ZeroAddress,
                    ethers.parseEther("0.5")
                )).to.be.revertedWith("Cannot recover staked token");
            } else {
                // ETH is not a staked token, so recovery should succeed
                await expect(stakePool.emergencyRecoverToken(
                    ethers.ZeroAddress,
                    ethers.parseEther("0.5")
                )).to.not.be.reverted;
            }
        });
    });
    
    describe("Edge Cases and Error Handling", function () {
        async function setupEdgeCaseFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            return fixture;
        }
        
        it("Should handle updatePoolReward with no staked tokens", async function () {
            const { stakePool } = await loadFixture(setupEdgeCaseFixture);
            
            // Call updatePoolReward when pool is empty
            await expect(stakePool.updatePoolReward(0)).to.not.be.reverted;
            
            const pool = await stakePool.pools(0);
            expect(pool.stTokenAmount).to.equal(0);
        });
        
        it("Should handle updatePoolReward when lastRewardBlock is current block", async function () {
            const { stakePool, user1 } = await loadFixture(setupEdgeCaseFixture);
            
            // Stake to initialize pool
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Call updatePoolReward in same block
            await expect(stakePool.updatePoolReward(0)).to.not.be.reverted;
        });
        
        it("Should handle invalid pool ID", async function () {
            const { stakePool, user1 } = await loadFixture(setupEdgeCaseFixture);
            
            // Try to stake in non-existent pool
            await expect(stakePool.connect(user1).stake(999, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            })).to.be.revertedWith("Invalid pool ID");
        });
        
        it("Should handle inactive pool", async function () {
            const { stakePool, user1 } = await loadFixture(setupEdgeCaseFixture);
            
            // Deactivate pool
            await stakePool.setPoolActive(0, false);
            
            // Try to stake in inactive pool
            await expect(stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            })).to.be.revertedWith("Pool is not active");
        });
        
        it("Should handle zero unstake amount", async function () {
            const { stakePool, user1 } = await loadFixture(setupEdgeCaseFixture);
            
            // Stake first
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Try to unstake zero amount
            await expect(stakePool.connect(user1).unstake(0, 0))
                .to.be.revertedWith("Amount must be greater than 0");
        });
        
        it("Should handle emergencyWithdraw with no staked amount", async function () {
            const { stakePool, user1 } = await loadFixture(setupEdgeCaseFixture);
            
            // Try to emergency withdraw without staking
            await expect(stakePool.connect(user1).emergencyWithdraw(0))
                .to.be.revertedWith("No staked amount");
        });
        
        it("Should handle ERC20 staking with ETH sent", async function () {
            const { stakePool, testToken, user1 } = await loadFixture(deployStakePoolFixture);
            
            // Add ERC20 pool
            await stakePool.addPool(
                await testToken.getAddress(),
                100,
                ethers.parseEther("10"),
                100
            );
            
            // Try to stake ERC20 but send ETH (pool index 0)
            await expect(stakePool.connect(user1).stake(0, ethers.parseEther("10"), { 
                value: ethers.parseEther("1") 
            })).to.be.revertedWith("Should not send ETH for ERC20 token");
        });
    });
    
    describe("Multiple Operations Scenarios", function () {
        async function setupMultiOpFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            // Add multiple pools with different weights
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                50
            );
            
            await fixture.stakePool.addPool(
                await fixture.testToken.getAddress(),
                200,
                ethers.parseEther("10"),
                100
            );
            
            await fixture.stakePool.addPool(
                await fixture.testToken.getAddress(),
                50,
                ethers.parseEther("5"),
                150
            );
            
            return fixture;
        }
        
        it("Should handle complex multi-pool operations", async function () {
            const { stakePool, testToken, user1, user2 } = await loadFixture(setupMultiOpFixture);
            
            // User1 stakes in all pools
            await stakePool.connect(user1).stake(0, ethers.parseEther("2"), { 
                value: ethers.parseEther("2") 
            });
            
            await testToken.connect(user1).approve(await stakePool.getAddress(), ethers.parseEther("500"));
            await stakePool.connect(user1).stake(1, ethers.parseEther("100"));
            await stakePool.connect(user1).stake(2, ethers.parseEther("50"));
            
            // User2 stakes in pool 0 and 1
            await stakePool.connect(user2).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            await testToken.connect(user2).approve(await stakePool.getAddress(), ethers.parseEther("200"));
            await stakePool.connect(user2).stake(1, ethers.parseEther("200"));
            
            // Mine blocks to accumulate rewards
            for (let i = 0; i < 20; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            // Check that all pools have correct staked amounts
            const pool0 = await stakePool.pools(0);
            const pool1 = await stakePool.pools(1);
            const pool2 = await stakePool.pools(2);
            
            expect(pool0.stTokenAmount).to.equal(ethers.parseEther("3"));
            expect(pool1.stTokenAmount).to.equal(ethers.parseEther("300"));
            expect(pool2.stTokenAmount).to.equal(ethers.parseEther("50"));
            
            // Check total pool weight
            expect(await stakePool.totalPoolWeight()).to.equal(350); // 100 + 200 + 50
        });
        
        it("Should handle partial unstaking from multiple pools", async function () {
            const { stakePool, testToken, user1 } = await loadFixture(setupMultiOpFixture);
            
            // Stake in multiple pools
            await stakePool.connect(user1).stake(0, ethers.parseEther("2"), { 
                value: ethers.parseEther("2") 
            });
            
            await testToken.connect(user1).approve(await stakePool.getAddress(), ethers.parseEther("200"));
            await stakePool.connect(user1).stake(1, ethers.parseEther("100"));
            await stakePool.connect(user1).stake(2, ethers.parseEther("50"));
            
            // Partial unstake from each pool
            await stakePool.connect(user1).unstake(0, ethers.parseEther("0.5"));
            await stakePool.connect(user1).unstake(1, ethers.parseEther("30"));
            await stakePool.connect(user1).unstake(2, ethers.parseEther("20"));
            
            // Check user info
            const user0Info = await stakePool.getUserInfo(0, user1.address);
            const user1Info = await stakePool.getUserInfo(1, user1.address);
            const user2Info = await stakePool.getUserInfo(2, user1.address);
            
            expect(user0Info.stAmount).to.equal(ethers.parseEther("1.5"));
            expect(user1Info.stAmount).to.equal(ethers.parseEther("70"));
            expect(user2Info.stAmount).to.equal(ethers.parseEther("30"));
            
            // Each should have one unstake request
            expect(user0Info.requests).to.have.length(1);
            expect(user1Info.requests).to.have.length(1);
            expect(user2Info.requests).to.have.length(1);
        });
    });
});