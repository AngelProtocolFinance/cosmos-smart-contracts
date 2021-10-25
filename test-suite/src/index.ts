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
		case "localterra_setup_terraswap":
			await LocalNet.startSetupTerraSwapContracts();
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
		case "testnet":
			await TestNet.startTest();
			break;
		case "testnet_setup":
			await TestNet.startSetupContracts();
			break;
		case "testnet_setup_terraswap":
			await TestNet.startSetupTerraSwapContracts();
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
		case "mainnet":
			await MainNet.startTest();
			break;
		case "mainnet_setup":
			await MainNet.startSetupContracts();
			break;
		case "mainnet_setup_terraswap":
			await MainNet.startSetupTerraSwapContracts();
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
		default:
			console.log("Invalid command");
			break;
	}
})();
