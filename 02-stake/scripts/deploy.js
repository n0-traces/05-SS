const { ethers, upgrades } = require("hardhat");

async function main() {
    const [deployer] = await ethers.getSigners();
    
    console.log("Deploying contracts with the account:", deployer.address);
    console.log("Account balance:", (await ethers.provider.getBalance(deployer.address)).toString());
    
    // Deploy MetaNodeToken
    console.log("\n1. Deploying MetaNodeToken...");
    const MetaNodeToken = await ethers.getContractFactory("MetaNodeToken");
    const metaNodeToken = await MetaNodeToken.deploy();
    await metaNodeToken.waitForDeployment();
    const metaNodeTokenAddress = await metaNodeToken.getAddress();
    console.log("MetaNodeToken deployed to:", metaNodeTokenAddress);
    
    // Deploy TestToken for testing (optional)
    console.log("\n2. Deploying TestToken for testing...");
    const TestToken = await ethers.getContractFactory("TestToken");
    const testToken = await TestToken.deploy("Test Stake Token", "TST", 18, 1000000);
    await testToken.waitForDeployment();
    const testTokenAddress = await testToken.getAddress();
    console.log("TestToken deployed to:", testTokenAddress);
    
    // Deploy StakePool as upgradeable proxy
    console.log("\n3. Deploying StakePool...");
    const StakePool = await ethers.getContractFactory("StakePool");
    
    const metaNodePerBlock = ethers.parseEther("1"); // 1 META per block
    const startBlock = await ethers.provider.getBlockNumber() + 100; // Start in 100 blocks
    
    const stakePool = await upgrades.deployProxy(
        StakePool,
        [metaNodeTokenAddress, metaNodePerBlock, startBlock],
        { initializer: 'initialize' }
    );
    await stakePool.waitForDeployment();
    const stakePoolAddress = await stakePool.getAddress();
    console.log("StakePool deployed to:", stakePoolAddress);
    
    // Transfer some META tokens to StakePool for rewards
    console.log("\n4. Setting up initial token distribution...");
    const rewardAmount = ethers.parseEther("100000"); // 100k META tokens
    await metaNodeToken.transfer(stakePoolAddress, rewardAmount);
    console.log("Transferred", ethers.formatEther(rewardAmount), "META tokens to StakePool for rewards");
    
    // Add initial pools
    console.log("\n5. Adding initial pools...");
    
    // Pool 0: Native ETH staking
    const ethPoolWeight = 100;
    const ethMinDeposit = ethers.parseEther("0.01"); // 0.01 ETH minimum
    const ethLockBlocks = 6500; // ~24 hours at 13s per block
    
    await stakePool.addPool(
        ethers.ZeroAddress, // Native ETH
        ethPoolWeight,
        ethMinDeposit,
        ethLockBlocks
    );
    console.log("Added ETH staking pool (Pool 0)");
    
    // Pool 1: TestToken staking
    const testTokenPoolWeight = 200;
    const testTokenMinDeposit = ethers.parseEther("100"); // 100 TEST tokens minimum
    const testTokenLockBlocks = 13000; // ~48 hours
    
    await stakePool.addPool(
        testTokenAddress,
        testTokenPoolWeight,
        testTokenMinDeposit,
        testTokenLockBlocks
    );
    console.log("Added TestToken staking pool (Pool 1)");
    
    // Verify deployment
    console.log("\n6. Verifying deployment...");
    const poolLength = await stakePool.getPoolLength();
    const totalWeight = await stakePool.totalPoolWeight();
    const metaPerBlock = await stakePool.metaNodePerBlock();
    
    console.log("Pool count:", poolLength.toString());
    console.log("Total pool weight:", totalWeight.toString());
    console.log("META per block:", ethers.formatEther(metaPerBlock));
    
    // Save deployment info
    const deploymentInfo = {
        network: "sepolia",
        deployer: deployer.address,
        blockNumber: await ethers.provider.getBlockNumber(),
        timestamp: new Date().toISOString(),
        contracts: {
            MetaNodeToken: {
                address: metaNodeTokenAddress,
                name: "MetaNode Token",
                symbol: "META"
            },
            TestToken: {
                address: testTokenAddress,
                name: "Test Stake Token",
                symbol: "TST"
            },
            StakePool: {
                address: stakePoolAddress,
                metaNodePerBlock: ethers.formatEther(metaPerBlock),
                startBlock: startBlock.toString(),
                pools: [
                    {
                        id: 0,
                        token: "ETH",
                        address: ethers.ZeroAddress,
                        weight: ethPoolWeight,
                        minDeposit: ethers.formatEther(ethMinDeposit),
                        lockBlocks: ethLockBlocks
                    },
                    {
                        id: 1,
                        token: "TST",
                        address: testTokenAddress,
                        weight: testTokenPoolWeight,
                        minDeposit: ethers.formatEther(testTokenMinDeposit),
                        lockBlocks: testTokenLockBlocks
                    }
                ]
            }
        }
    };
    
    const fs = require('fs');
    const path = require('path');
    
    const deploymentDir = path.join(__dirname, '../deployments');
    if (!fs.existsSync(deploymentDir)) {
        fs.mkdirSync(deploymentDir, { recursive: true });
    }
    
    const deploymentFile = path.join(deploymentDir, `sepolia-${Date.now()}.json`);
    fs.writeFileSync(deploymentFile, JSON.stringify(deploymentInfo, null, 2));
    
    console.log("\n✅ Deployment completed successfully!");
    console.log("📝 Deployment info saved to:", deploymentFile);
    console.log("\n📋 Contract Addresses:");
    console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    console.log("MetaNodeToken:", metaNodeTokenAddress);
    console.log("TestToken:", testTokenAddress);
    console.log("StakePool:", stakePoolAddress);
    console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    return {
        metaNodeToken: metaNodeTokenAddress,
        testToken: testTokenAddress,
        stakePool: stakePoolAddress
    };
}

// Allow script to be called directly
if (require.main === module) {
    main()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}

module.exports = main;