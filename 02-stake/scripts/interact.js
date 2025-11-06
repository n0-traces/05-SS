const { ethers } = require("hardhat");

async function main() {
    const [deployer, user1] = await ethers.getSigners();
    
    // Contract addresses - update these after deployment
    const STAKE_POOL_ADDRESS = process.env.STAKE_POOL_ADDRESS;
    const META_TOKEN_ADDRESS = process.env.META_TOKEN_ADDRESS;
    const TEST_TOKEN_ADDRESS = process.env.TEST_TOKEN_ADDRESS;
    
    if (!STAKE_POOL_ADDRESS || !META_TOKEN_ADDRESS || !TEST_TOKEN_ADDRESS) {
        console.error("Please set contract addresses in environment variables:");
        console.error("STAKE_POOL_ADDRESS, META_TOKEN_ADDRESS, TEST_TOKEN_ADDRESS");
        process.exit(1);
    }
    
    // Get contract instances
    const stakePool = await ethers.getContractAt("StakePool", STAKE_POOL_ADDRESS);
    const metaToken = await ethers.getContractAt("MetaNodeToken", META_TOKEN_ADDRESS);
    const testToken = await ethers.getContractAt("TestToken", TEST_TOKEN_ADDRESS);
    
    console.log("🔗 Connected to contracts:");
    console.log("StakePool:", STAKE_POOL_ADDRESS);
    console.log("MetaToken:", META_TOKEN_ADDRESS);
    console.log("TestToken:", TEST_TOKEN_ADDRESS);
    
    // Display current status
    console.log("\n📊 Current Status:");
    const poolLength = await stakePool.getPoolLength();
    const totalWeight = await stakePool.totalPoolWeight();
    const metaPerBlock = await stakePool.metaNodePerBlock();
    
    console.log("Pool count:", poolLength.toString());
    console.log("Total pool weight:", totalWeight.toString());
    console.log("META per block:", ethers.formatEther(metaPerBlock));
    
    // Display pool information
    for (let i = 0; i < poolLength; i++) {
        const pool = await stakePool.pools(i);
        console.log(`\nPool ${i}:`);
        console.log("  Token address:", pool.stTokenAddress);
        console.log("  Weight:", pool.poolWeight.toString());
        console.log("  Min deposit:", ethers.formatEther(pool.minDepositAmount));
        console.log("  Lock blocks:", pool.unstakeLockedBlocks.toString());
        console.log("  Total staked:", ethers.formatEther(pool.stTokenAmount));
        console.log("  Active:", pool.isActive);
    }
    
    // Example interactions
    console.log("\n🚀 Running example interactions...");
    
    // 1. Stake ETH in pool 0
    console.log("\n1. Staking 0.1 ETH in Pool 0...");
    const ethStakeAmount = ethers.parseEther("0.1");
    
    try {
        const tx1 = await stakePool.connect(user1).stake(0, ethStakeAmount, { 
            value: ethStakeAmount 
        });
        await tx1.wait();
        console.log("✅ ETH staking successful, tx:", tx1.hash);
        
        const userInfo1 = await stakePool.getUserInfo(0, user1.address);
        console.log("User staked amount:", ethers.formatEther(userInfo1.stAmount), "ETH");
        
    } catch (error) {
        console.log("❌ ETH staking failed:", error.message);
    }
    
    // 2. Get test tokens and stake in pool 1
    console.log("\n2. Getting test tokens and staking in Pool 1...");
    
    try {
        // Mint test tokens to user1
        const testTokenAmount = ethers.parseEther("1000");
        const tx2 = await testToken.mint(user1.address, testTokenAmount);
        await tx2.wait();
        console.log("✅ Minted", ethers.formatEther(testTokenAmount), "TEST tokens to user1");
        
        // Approve spending
        const stakeAmount = ethers.parseEther("500");
        const tx3 = await testToken.connect(user1).approve(STAKE_POOL_ADDRESS, stakeAmount);
        await tx3.wait();
        console.log("✅ Approved TEST token spending");
        
        // Stake tokens
        const tx4 = await stakePool.connect(user1).stake(1, stakeAmount);
        await tx4.wait();
        console.log("✅ TEST token staking successful, tx:", tx4.hash);
        
        const userInfo2 = await stakePool.getUserInfo(1, user1.address);
        console.log("User staked amount:", ethers.formatEther(userInfo2.stAmount), "TEST");
        
    } catch (error) {
        console.log("❌ TEST token staking failed:", error.message);
    }
    
    // 3. Wait for some rewards to accumulate
    console.log("\n3. Mining blocks to accumulate rewards...");
    
    for (let i = 0; i < 10; i++) {
        await ethers.provider.send("evm_mine");
    }
    
    // Check pending rewards
    try {
        const pendingEth = await stakePool.pendingMetaNode(0, user1.address);
        const pendingTest = await stakePool.pendingMetaNode(1, user1.address);
        
        console.log("Pending ETH rewards:", ethers.formatEther(pendingEth), "META");
        console.log("Pending TEST rewards:", ethers.formatEther(pendingTest), "META");
        
    } catch (error) {
        console.log("❌ Failed to get pending rewards:", error.message);
    }
    
    // 4. Claim rewards
    console.log("\n4. Claiming rewards...");
    
    try {
        const balanceBefore = await metaToken.balanceOf(user1.address);
        
        const tx5 = await stakePool.connect(user1).claim(0);
        await tx5.wait();
        console.log("✅ Claimed ETH pool rewards, tx:", tx5.hash);
        
        const tx6 = await stakePool.connect(user1).claim(1);
        await tx6.wait();
        console.log("✅ Claimed TEST pool rewards, tx:", tx6.hash);
        
        const balanceAfter = await metaToken.balanceOf(user1.address);
        const claimed = balanceAfter - balanceBefore;
        
        console.log("Total META claimed:", ethers.formatEther(claimed));
        
    } catch (error) {
        console.log("❌ Failed to claim rewards:", error.message);
    }
    
    // 5. Test unstaking
    console.log("\n5. Testing unstaking...");
    
    try {
        const unstakeAmount = ethers.parseEther("0.05"); // Unstake half of ETH
        
        const tx7 = await stakePool.connect(user1).unstake(0, unstakeAmount);
        await tx7.wait();
        console.log("✅ Unstaked", ethers.formatEther(unstakeAmount), "ETH, tx:", tx7.hash);
        
        const userInfo3 = await stakePool.getUserInfo(0, user1.address);
        console.log("Remaining staked:", ethers.formatEther(userInfo3.stAmount), "ETH");
        console.log("Unstake requests:", userInfo3.requests.length);
        
        if (userInfo3.requests.length > 0) {
            const request = userInfo3.requests[0];
            console.log("First request amount:", ethers.formatEther(request.amount), "ETH");
            console.log("Unlock block:", request.unlockBlock.toString());
            console.log("Current block:", (await ethers.provider.getBlockNumber()).toString());
        }
        
    } catch (error) {
        console.log("❌ Failed to unstake:", error.message);
    }
    
    // 6. Check withdrawable amount
    console.log("\n6. Checking withdrawable amounts...");
    
    try {
        const withdrawableEth = await stakePool.getWithdrawableAmount(0, user1.address);
        const withdrawableTest = await stakePool.getWithdrawableAmount(1, user1.address);
        
        console.log("Withdrawable ETH:", ethers.formatEther(withdrawableEth));
        console.log("Withdrawable TEST:", ethers.formatEther(withdrawableTest));
        
    } catch (error) {
        console.log("❌ Failed to get withdrawable amounts:", error.message);
    }
    
    console.log("\n✅ Interaction script completed!");
    console.log("\n💡 To withdraw unstaked tokens, wait for the lock period to pass,");
    console.log("   then call the withdraw() function for the respective pool.");
}

if (require.main === module) {
    main()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

module.exports = main;