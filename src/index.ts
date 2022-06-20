import * as LocalNet from "./environments/localjuno";
import * as TestNet from "./environments/testnet";
import * as MainNet from "./environments/mainnet";

//----------------------------------------------------------------------------------------
// Test-suite for  TestNet, and MainNet
//----------------------------------------------------------------------------------------
(async () => {
	const mode = process.env.npm_config_mode || "";
	switch (mode) {
		// case "localjuno_tests":
		// 	await LocalNet.startTests();
		// 	break;
		case "localjuno_setup_core":
			await LocalNet.startSetupCore();
			break;
		// case "localjuno_setup_halo":
		// 	await LocalNet.startSetupHalo();
		// 	break;
		// case "localjuno_setup_terraswap":
		// 	await LocalNet.startSetupJunoSwap();
		// 	break;
		// case "localjuno_migrate_core":
		// 	await LocalNet.startMigrateCore();
		// 	break;
		// case "localjuno_migrate_halo":
		// 	await LocalNet.startMigrateHalo();
		// 	break;
		// case "testnet_tests":
		// 	await TestNet.startTests();
		// 	break;
		case "testnet_setup_core":
			await TestNet.startSetupCore();
			break;
		// case "testnet_setup_halo":
		// 	await TestNet.startSetupHalo();
		// 	break;
		// case "testnet_setup_terraswap":
		// 	await TestNet.startSetupTerraSwap();
		// 	break;
		// case "testnet_migrate_core":
		// 	await TestNet.startMigrateCore();
		// 	break;
		// case "testnet_migrate_halo":
		// 	await TestNet.startMigrateHalo();
		// 	break;
		// case "mainnet_tests":
		// 	await MainNet.startTests();
		// 	break;
		// case "mainnet_setup_core":
		// 	await MainNet.startSetupCore();
		// 	break;
		// case "mainnet_setup_halo":
		// 	await MainNet.startSetupHalo();
		// 	break;
		// case "mainnet_setup_terraswap":
		// 	await MainNet.startSetupTerraSwap();
		// 	break;
		// case "mainnet_migrate_core":
		// 	await MainNet.startMigrateCore();
		// 	break;
		// case "mainnet_migrate_halo":
		// 	await MainNet.startMigrateHalo();
		// 	break;
		default:
			console.log("Invalid command");
			break;
	}
})();
