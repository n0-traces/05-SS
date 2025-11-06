const { ethers, upgrades } = require("hardhat");

async function main() {
    const [deployer] = await ethers.getSigners();
    
    console.log("Upgrading contracts with the account:", deployer.address);
    
    // You need to provide the existing proxy address here
    const PROXY_ADDRESS = process.env.STAKE_POOL_PROXY_ADDRESS;
    
    if (!PROXY_ADDRESS) {
        console.error("Please set STAKE_POOL_PROXY_ADDRESS environment variable");
        process.exit(1);
    }
    
    console.log("Upgrading StakePool proxy at:", PROXY_ADDRESS);
    
    // Deploy the new implementation
    console.log("\n1. Deploying new implementation (StakePoolV2)...");
    const StakePoolV2 = await ethers.getContractFactory("StakePoolV2");
    
    const upgraded = await upgrades.upgradeProxy(PROXY_ADDRESS, StakePoolV2);
    await upgraded.waitForDeployment();
    
    console.log("StakePool upgraded successfully");
    
    // Test the upgrade by calling a new function
    console.log("\n2. Testing upgrade...");
    
    try {
        const version = await upgraded.version();
        console.log("Contract version:", version);
        
        const bonusMultiplier = await upgraded.bonusMultiplier();
        console.log("Bonus multiplier:", bonusMultiplier.toString());
        
        console.log("✅ Upgrade completed and verified!");
        
    } catch (error) {
        console.error("❌ Upgrade verification failed:", error.message);
    }
    
    // Save upgrade info
    const upgradeInfo = {
        network: "sepolia",
        upgrader: deployer.address,
        blockNumber: await ethers.provider.getBlockNumber(),
        timestamp: new Date().toISOString(),
        proxyAddress: PROXY_ADDRESS,
        newImplementation: "StakePoolV2",
        version: "2.0.0"
    };
    
    const fs = require('fs');
    const path = require('path');
    
    const upgradeFile = path.join(__dirname, '../deployments', `upgrade-${Date.now()}.json`);
    fs.writeFileSync(upgradeFile, JSON.stringify(upgradeInfo, null, 2));
    
    console.log("📝 Upgrade info saved to:", upgradeFile);
}

if (require.main === module) {
    main()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error(error);
            process.exit(1);
        });
}