import * as LocalNet from "./environments/localterra";
import * as TestNet from "./environments/testnet";
import * as MainNet from "./environments/mainnet";

//----------------------------------------------------------------------------------------
// Test-suite for LocalTerra, TestNet, and MainNet
//----------------------------------------------------------------------------------------
(async () => {
	const mode = process.env.npm_config_mode || "";
	switch (mode) {
		case "localterra":
			await LocalNet.startTest();
			break;
		case "localterra_setup":
			await LocalNet.startSetupContracts();
			break;
		case "localterra_setup_lbp":
			await LocalNet.startSetupLBPContracts();
			break;
		case "localterra_setup_halo":
			await LocalNet.startSetupHalo();
			break;
		case "localterra_migrate":
			await LocalNet.startMigrateContracts();
			break;
		case "localterra_migrate_halo":
			await LocalNet.startMigrateHaloContracts();
			break;
		case "localterra_migrate_lbp":
			await LocalNet.startMigrateLBPContracts();
			break;
		case "testnet":
			await TestNet.startTest();
			break;
		case "testnet_setup":
			await TestNet.startSetupContracts();
			break;
		case "testnet_setup_lbp":
			await TestNet.startSetupLBPContracts();
			break;
		case "testnet_setup_halo":
			await TestNet.startSetupHalo();
			break;
		case "testnet_migrate":
			await TestNet.startMigrateContracts();
			break;
		case "testnet_migrate_halo":
			await TestNet.startMigrateHaloContracts();
			break;
		case "testnet_migrate_lbp":
			await TestNet.startMigrateLBPContracts();
			break;
		case "mainnet":
			await MainNet.startTest();
			break;
		case "mainnet_setup":
			await MainNet.startSetupContracts();
			break;
		case "mainnet_setup_lbp":
			await MainNet.startSetupLBPContracts();
			break;
		case "mainnet_setup_halo":
			await MainNet.startSetupHalo();
			break;
		case "mainnet_migrate":
			await MainNet.startMigrateContracts();
			break;
		case "mainnet_migrate_halo":
			await MainNet.startMigrateHaloContracts();
			break;
		case "mainnet_migrate_lbp":
			await MainNet.startMigrateLBPContracts();
			break;
		default:
			console.log("Invalid command");
			break;
	}
})();
