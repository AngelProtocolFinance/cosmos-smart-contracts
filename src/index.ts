import * as LocalNet from "./environments/localterra";
import * as TestNet from "./environments/testnet";
import * as MainNet from "./environments/mainnet";

//----------------------------------------------------------------------------------------
// Test-suite for LocalTerra, TestNet, and MainNet
//----------------------------------------------------------------------------------------
(async () => {
	const mode = process.env.npm_config_mode || "";
	switch (mode) {
		case "localterra_tests":
			await LocalNet.startTests();
			break;
		case "localterra_setup_core":
			await LocalNet.startSetupCore();
			break;
		case "localterra_setup_lbp":
			await LocalNet.startSetupLbp();
			break;
		case "localterra_setup_halo":
			await LocalNet.startSetupHalo();
			break;
		case "localterra_setup_terraswap":
			await LocalNet.startSetupTerraSwap();
			break;
		case "localterra_migrate_core":
			await LocalNet.startMigrateCore();
			break;
		case "localterra_migrate_halo":
			await LocalNet.startMigrateHalo();
			break;
		case "localterra_migrate_lbp":
			await LocalNet.startMigrateLbp();
			break;
		case "testnet_tests":
			await TestNet.startTests();
			break;
		case "testnet_setup_core":
			await TestNet.startSetupCore();
			break;
		case "testnet_setup_lbp":
			await TestNet.startSetupLbp();
			break;
		case "testnet_setup_halo":
			await TestNet.startSetupHalo();
			break;
		case "testnet_setup_terraswap":
			await TestNet.startSetupTerraSwap();
			break;
		case "testnet_migrate_core":
			await TestNet.startMigrateCore();
			break;
		case "testnet_migrate_halo":
			await TestNet.startMigrateHalo();
			break;
		case "testnet_migrate_lbp":
			await TestNet.startMigrateLbp();
			break;
		case "mainnet_tests":
			await MainNet.startTests();
			break;
		case "mainnet_setup_core":
			await MainNet.startSetupCore();
			break;
		case "mainnet_setup_lbp":
			await MainNet.startSetupLbp();
			break;
		case "mainnet_setup_halo":
			await MainNet.startSetupHalo();
			break;
		case "mainnet_setup_terraswap":
			await MainNet.startSetupTerraSwap();
			break;
		case "mainnet_migrate_core":
			await MainNet.startMigrateCore();
			break;
		case "mainnet_migrate_halo":
			await MainNet.startMigrateHalo();
			break;
		case "mainnet_migrate_lbp":
			await MainNet.startMigrateLbp();
			break;
		default:
			console.log("Invalid command");
			break;
	}
})();
