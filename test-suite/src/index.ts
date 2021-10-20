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
		case "localterra_migrate":
			await LocalNet.startMigrateContracts();
			break;
		case "testnet":
			await TestNet.startTest();
			break;
		case "testnet_setup":
			await TestNet.startSetupContracts();
			break;
		case "testnet_migrate":
			await TestNet.startMigrateContracts();
			break;
		case "mainnet":
			await MainNet.startTest();
			break;
		case "mainnet_setup":
			await MainNet.startSetupContracts();
			break;
		case "mainnet_migrate":
			await MainNet.startMigrateContracts();
			break;
		default:
			console.log("Invalid command");
			break;
	}
})();
