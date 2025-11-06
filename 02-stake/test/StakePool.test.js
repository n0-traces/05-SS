const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("StakePool", function () {
    async function deployStakePoolFixture() {
        const [owner, user1, user2, user3] = await ethers.getSigners();
        
        // Deploy MetaNodeToken
        const MetaNodeToken = await ethers.getContractFactory("MetaNodeToken");
        const metaNodeToken = await MetaNodeToken.deploy();
        
        // Deploy TestToken for ERC20 staking
        const TestToken = await ethers.getContractFactory("TestToken");
        const testToken = await TestToken.deploy("Test Token", "TEST", 18, 1000000);
        
        // Deploy StakePool
        const StakePool = await ethers.getContractFactory("StakePool");
        const stakePool = await StakePool.deploy();
        
        const metaNodePerBlock = ethers.parseEther("10"); // 10 META per block
        const startBlock = await ethers.provider.getBlockNumber() + 1;
        
        await stakePool.initialize(
            await metaNodeToken.getAddress(),
            metaNodePerBlock,
            startBlock
        );
        
        // Transfer some META tokens to StakePool for rewards
        await metaNodeToken.transfer(
            await stakePool.getAddress(),
            ethers.parseEther("100000")
        );
        
        // Transfer test tokens to users
        await testToken.transfer(user1.address, ethers.parseEther("1000"));
        await testToken.transfer(user2.address, ethers.parseEther("1000"));
        await testToken.transfer(user3.address, ethers.parseEther("1000"));
        
        return {
            stakePool,
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
    
    describe("Deployment", function () {
        it("Should initialize with correct parameters", async function () {
            const { stakePool, metaNodeToken, metaNodePerBlock } = await loadFixture(deployStakePoolFixture);
            
            expect(await stakePool.metaNodeToken()).to.equal(await metaNodeToken.getAddress());
            expect(await stakePool.metaNodePerBlock()).to.equal(metaNodePerBlock);
            expect(await stakePool.totalPoolWeight()).to.equal(0);
        });
        
        it("Should grant admin roles to deployer", async function () {
            const { stakePool, owner } = await loadFixture(deployStakePoolFixture);
            
            const ADMIN_ROLE = await stakePool.ADMIN_ROLE();
            const UPGRADER_ROLE = await stakePool.UPGRADER_ROLE();
            const OPERATOR_ROLE = await stakePool.OPERATOR_ROLE();
            
            expect(await stakePool.hasRole(ADMIN_ROLE, owner.address)).to.be.true;
            expect(await stakePool.hasRole(UPGRADER_ROLE, owner.address)).to.be.true;
            expect(await stakePool.hasRole(OPERATOR_ROLE, owner.address)).to.be.true;
        });
    });
    
    describe("Pool Management", function () {
        it("Should add a native currency pool", async function () {
            const { stakePool } = await loadFixture(deployStakePoolFixture);
            
            const minDeposit = ethers.parseEther("0.1");
            const unstakeLockedBlocks = 100;
            const poolWeight = 100;
            
            await expect(stakePool.addPool(
                ethers.ZeroAddress, // Native currency
                poolWeight,
                minDeposit,
                unstakeLockedBlocks
            ))
                .to.emit(stakePool, "PoolAdded")
                .withArgs(0, ethers.ZeroAddress, poolWeight, minDeposit, unstakeLockedBlocks);
            
            const pool = await stakePool.pools(0);
            expect(pool.stTokenAddress).to.equal(ethers.ZeroAddress);
            expect(pool.poolWeight).to.equal(poolWeight);
            expect(pool.minDepositAmount).to.equal(minDeposit);
            expect(pool.unstakeLockedBlocks).to.equal(unstakeLockedBlocks);
            expect(pool.isActive).to.be.true;
            
            expect(await stakePool.totalPoolWeight()).to.equal(poolWeight);
        });
        
        it("Should add an ERC20 token pool", async function () {
            const { stakePool, testToken } = await loadFixture(deployStakePoolFixture);
            
            const minDeposit = ethers.parseEther("10");
            const unstakeLockedBlocks = 200;
            const poolWeight = 200;
            
            await expect(stakePool.addPool(
                await testToken.getAddress(),
                poolWeight,
                minDeposit,
                unstakeLockedBlocks
            ))
                .to.emit(stakePool, "PoolAdded")
                .withArgs(0, await testToken.getAddress(), poolWeight, minDeposit, unstakeLockedBlocks);
            
            const pool = await stakePool.pools(0);
            expect(pool.stTokenAddress).to.equal(await testToken.getAddress());
        });
        
        it("Should update pool parameters", async function () {
            const { stakePool, testToken } = await loadFixture(deployStakePoolFixture);
            
            // Add pool first
            await stakePool.addPool(
                await testToken.getAddress(),
                100,
                ethers.parseEther("10"),
                200
            );
            
            // Update pool
            const newWeight = 150;
            const newMinDeposit = ethers.parseEther("20");
            const newLockedBlocks = 300;
            
            await expect(stakePool.updatePool(0, newWeight, newMinDeposit, newLockedBlocks))
                .to.emit(stakePool, "PoolUpdated")
                .withArgs(0, newWeight, newMinDeposit, newLockedBlocks);
            
            const pool = await stakePool.pools(0);
            expect(pool.poolWeight).to.equal(newWeight);
            expect(pool.minDepositAmount).to.equal(newMinDeposit);
            expect(pool.unstakeLockedBlocks).to.equal(newLockedBlocks);
        });
        
        it("Should not allow non-admin to manage pools", async function () {
            const { stakePool, user1, testToken } = await loadFixture(deployStakePoolFixture);
            
            await expect(stakePool.connect(user1).addPool(
                await testToken.getAddress(),
                100,
                ethers.parseEther("10"),
                200
            )).to.be.reverted;
        });
    });
    
    describe("Staking", function () {
        async function setupPoolsFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            // Add native currency pool (pool 0)
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            // Add ERC20 token pool (pool 1)
            await fixture.stakePool.addPool(
                await fixture.testToken.getAddress(),
                200,
                ethers.parseEther("10"),
                200
            );
            
            return fixture;
        }
        
        it("Should stake native currency", async function () {
            const { stakePool, user1 } = await loadFixture(setupPoolsFixture);
            
            const stakeAmount = ethers.parseEther("1");
            
            await expect(stakePool.connect(user1).stake(0, stakeAmount, { value: stakeAmount }))
                .to.emit(stakePool, "Deposit")
                .withArgs(user1.address, 0, stakeAmount);
            
            const userInfo = await stakePool.getUserInfo(0, user1.address);
            expect(userInfo.stAmount).to.equal(stakeAmount);
            
            const pool = await stakePool.pools(0);
            expect(pool.stTokenAmount).to.equal(stakeAmount);
        });
        
        it("Should stake ERC20 tokens", async function () {
            const { stakePool, testToken, user1 } = await loadFixture(setupPoolsFixture);
            
            const stakeAmount = ethers.parseEther("100");
            
            // Approve tokens first
            await testToken.connect(user1).approve(await stakePool.getAddress(), stakeAmount);
            
            await expect(stakePool.connect(user1).stake(1, stakeAmount))
                .to.emit(stakePool, "Deposit")
                .withArgs(user1.address, 1, stakeAmount);
            
            const userInfo = await stakePool.getUserInfo(1, user1.address);
            expect(userInfo.stAmount).to.equal(stakeAmount);
        });
        
        it("Should fail to stake below minimum deposit", async function () {
            const { stakePool, user1 } = await loadFixture(setupPoolsFixture);
            
            const stakeAmount = ethers.parseEther("0.05"); // Below minimum 0.1 ETH
            
            await expect(stakePool.connect(user1).stake(0, stakeAmount, { value: stakeAmount }))
                .to.be.revertedWith("Amount below minimum deposit");
        });
        
        it("Should fail to stake with incorrect ETH amount", async function () {
            const { stakePool, user1 } = await loadFixture(setupPoolsFixture);
            
            const stakeAmount = ethers.parseEther("1");
            const wrongEthAmount = ethers.parseEther("0.5");
            
            await expect(stakePool.connect(user1).stake(0, stakeAmount, { value: wrongEthAmount }))
                .to.be.revertedWith("Invalid ETH amount");
        });
    });
    
    describe("Unstaking and Withdrawal", function () {
        async function setupStakedFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            // Add pools
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                10 // Short lock period for testing
            );
            
            await fixture.stakePool.addPool(
                await fixture.testToken.getAddress(),
                200,
                ethers.parseEther("10"),
                20
            );
            
            // Stake some tokens
            await fixture.stakePool.connect(fixture.user1).stake(0, ethers.parseEther("2"), { 
                value: ethers.parseEther("2") 
            });
            
            await fixture.testToken.connect(fixture.user1).approve(
                await fixture.stakePool.getAddress(),
                ethers.parseEther("200")
            );
            await fixture.stakePool.connect(fixture.user1).stake(1, ethers.parseEther("200"));
            
            return fixture;
        }
        
        it("Should unstake tokens", async function () {
            const { stakePool, user1 } = await loadFixture(setupStakedFixture);
            
            const unstakeAmount = ethers.parseEther("1");
            
            await expect(stakePool.connect(user1).unstake(0, unstakeAmount))
                .to.emit(stakePool, "Unstake")
                .withArgs(user1.address, 0, unstakeAmount);
            
            const userInfo = await stakePool.getUserInfo(0, user1.address);
            expect(userInfo.stAmount).to.equal(ethers.parseEther("1")); // 2 - 1
            expect(userInfo.requests).to.have.length(1);
            expect(userInfo.requests[0].amount).to.equal(unstakeAmount);
        });
        
        it("Should withdraw after lock period", async function () {
            const { stakePool, user1 } = await loadFixture(setupStakedFixture);
            
            const unstakeAmount = ethers.parseEther("1");
            
            // Unstake
            await stakePool.connect(user1).unstake(0, unstakeAmount);
            
            // Mine blocks to pass lock period
            for (let i = 0; i < 15; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const balanceBefore = await ethers.provider.getBalance(user1.address);
            
            await expect(stakePool.connect(user1).withdraw(0))
                .to.emit(stakePool, "Withdraw")
                .withArgs(user1.address, 0, unstakeAmount);
            
            const balanceAfter = await ethers.provider.getBalance(user1.address);
            expect(balanceAfter).to.be.gt(balanceBefore);
            
            // Check that request was cleared
            const userInfo = await stakePool.getUserInfo(0, user1.address);
            expect(userInfo.requests).to.have.length(0);
        });
        
        it("Should not withdraw before lock period", async function () {
            const { stakePool, user1 } = await loadFixture(setupStakedFixture);
            
            const unstakeAmount = ethers.parseEther("1");
            
            // Unstake
            await stakePool.connect(user1).unstake(0, unstakeAmount);
            
            await expect(stakePool.connect(user1).withdraw(0))
                .to.be.revertedWith("No withdrawable amount");
        });
        
        it("Should handle multiple unstake requests", async function () {
            const { stakePool, user1 } = await loadFixture(setupStakedFixture);
            
            // Make multiple unstake requests
            await stakePool.connect(user1).unstake(0, ethers.parseEther("0.5"));
            await stakePool.connect(user1).unstake(0, ethers.parseEther("0.3"));
            await stakePool.connect(user1).unstake(0, ethers.parseEther("0.2"));
            
            const userInfo = await stakePool.getUserInfo(0, user1.address);
            expect(userInfo.requests).to.have.length(3);
            expect(userInfo.stAmount).to.equal(ethers.parseEther("1")); // 2 - 1
        });
        
        it("Should emergency withdraw", async function () {
            const { stakePool, user1 } = await loadFixture(setupStakedFixture);
            
            const stakedAmount = ethers.parseEther("2");
            const balanceBefore = await ethers.provider.getBalance(user1.address);
            
            await expect(stakePool.connect(user1).emergencyWithdraw(0))
                .to.emit(stakePool, "Withdraw")
                .withArgs(user1.address, 0, stakedAmount);
            
            const balanceAfter = await ethers.provider.getBalance(user1.address);
            expect(balanceAfter).to.be.gt(balanceBefore);
            
            // Check user data is cleared
            const userInfo = await stakePool.getUserInfo(0, user1.address);
            expect(userInfo.stAmount).to.equal(0);
            expect(userInfo.requests).to.have.length(0);
        });
    });
    
    describe("Rewards", function () {
        async function setupRewardsFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            // Add pool
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            // Stake tokens
            await fixture.stakePool.connect(fixture.user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            return fixture;
        }
        
        it("Should calculate pending rewards correctly", async function () {
            const { stakePool, user1, metaNodePerBlock } = await loadFixture(setupRewardsFixture);
            
            // Mine some blocks
            for (let i = 0; i < 5; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const pendingRewards = await stakePool.pendingMetaNode(0, user1.address);
            
            // Should be approximately metaNodePerBlock * blocks (accounting for pool weight)
            expect(pendingRewards).to.be.gt(0);
        });
        
        it("Should claim rewards", async function () {
            const { stakePool, metaNodeToken, user1 } = await loadFixture(setupRewardsFixture);
            
            // Mine some blocks to accumulate rewards
            for (let i = 0; i < 10; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const balanceBefore = await metaNodeToken.balanceOf(user1.address);
            const pendingBefore = await stakePool.pendingMetaNode(0, user1.address);
            
            const tx = await stakePool.connect(user1).claim(0);
            const receipt = await tx.wait();
            
            // Find the Claim event
            const claimEvent = receipt.logs.find(log => {
                try {
                    const parsed = stakePool.interface.parseLog(log);
                    return parsed.name === "Claim";
                } catch {
                    return false;
                }
            });
            
            expect(claimEvent).to.not.be.undefined;
            const parsedEvent = stakePool.interface.parseLog(claimEvent);
            expect(parsedEvent.args[0]).to.equal(user1.address);
            expect(parsedEvent.args[1]).to.equal(0);
            
            const balanceAfter = await metaNodeToken.balanceOf(user1.address);
            expect(balanceAfter).to.be.gt(balanceBefore);
            
            // Pending should be reset to 0 (or very small due to block mining)
            const pendingAfter = await stakePool.pendingMetaNode(0, user1.address);
            expect(pendingAfter).to.be.lt(pendingBefore);
        });
        
        it("Should distribute rewards proportionally among users", async function () {
            const { stakePool, user1, user2 } = await loadFixture(setupRewardsFixture);
            
            // User2 stakes double the amount of user1
            await stakePool.connect(user2).stake(0, ethers.parseEther("2"), { 
                value: ethers.parseEther("2") 
            });
            
            // Mine some blocks
            for (let i = 0; i < 10; i++) {
                await ethers.provider.send("evm_mine");
            }
            
            const pending1 = await stakePool.pendingMetaNode(0, user1.address);
            const pending2 = await stakePool.pendingMetaNode(0, user2.address);
            
            // User2 should have approximately double the rewards of user1
            const ratio = pending2 * BigInt(100) / pending1;
            expect(ratio).to.be.approximately(200, 50); // Allow some tolerance
        });
    });
    
    describe("Pause Functionality", function () {
        async function setupPauseFixture() {
            const fixture = await loadFixture(deployStakePoolFixture);
            
            await fixture.stakePool.addPool(
                ethers.ZeroAddress,
                100,
                ethers.parseEther("0.1"),
                100
            );
            
            return fixture;
        }
        
        it("Should pause and unpause staking", async function () {
            const { stakePool, user1 } = await loadFixture(setupPauseFixture);
            
            // Pause staking
            await stakePool.setStakePaused(true);
            
            await expect(stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            })).to.be.revertedWith("Stake is paused");
            
            // Unpause staking
            await stakePool.setStakePaused(false);
            
            await expect(stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            })).to.emit(stakePool, "Deposit");
        });
        
        it("Should pause entire contract", async function () {
            const { stakePool, user1 } = await loadFixture(setupPauseFixture);
            
            // Stake first
            await stakePool.connect(user1).stake(0, ethers.parseEther("1"), { 
                value: ethers.parseEther("1") 
            });
            
            // Pause contract
            await stakePool.pause();
            
            // Should revert when paused (using custom error or reason string)
            await expect(stakePool.connect(user1).unstake(0, ethers.parseEther("0.5")))
                .to.be.reverted;
            
            // Unpause
            await stakePool.unpause();
            
            await expect(stakePool.connect(user1).unstake(0, ethers.parseEther("0.5")))
                .to.emit(stakePool, "Unstake");
        });
    });
    
    describe("Access Control", function () {
        it("Should not allow non-admin to set parameters", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await expect(stakePool.connect(user1).setMetaNodePerBlock(ethers.parseEther("20")))
                .to.be.reverted;
            
            await expect(stakePool.connect(user1).setPoolActive(0, false))
                .to.be.reverted;
        });
        
        it("Should not allow non-operator to pause", async function () {
            const { stakePool, user1 } = await loadFixture(deployStakePoolFixture);
            
            await expect(stakePool.connect(user1).pause())
                .to.be.reverted;
            
            await expect(stakePool.connect(user1).setStakePaused(true))
                .to.be.reverted;
        });
    });
});