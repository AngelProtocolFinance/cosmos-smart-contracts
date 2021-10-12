import * as LocalNet from "./environments/localterra";
import * as TestNet from "./environments/testnet";
import * as MainNet from "./environments/mainnet";

//----------------------------------------------------------------------------------------
// Test-suite for LocalTerra, TestNet, and MainNet
//----------------------------------------------------------------------------------------

function isValidMode(mode: string) {
  return mode === "LocalTerra" ||
    mode === "TestNet" ||
    mode === "MainNet";
}

(async () => {
	const mode = process.env.npm_config_mode || "";
	if (isValidMode(mode)) {
		if (mode === "LocalTerra") {
			await LocalNet.start();
		} else if (mode === "TestNet") {
				await TestNet.start();
		} else if (mode === "MainNet") {
			await MainNet.start();
		}
	} else {
		console.error("Invalid network");
	}
})();
