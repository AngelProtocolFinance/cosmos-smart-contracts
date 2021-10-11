import { LCDClient, LocalTerra } from "@terra-money/terra.js";
import * as LocalTest from "./local_terra";
import * as TestNet from "./testnet";
import * as MainNet from "./mainnet";

import {getNetworkInfo} from "../config";
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
function isValidMode(mode: string) {
  return mode === "LocalTerra" ||
    mode === "TestNet" ||
    mode === "MainNet";
}

(async () => {
	const mode = process.env.npm_config_mode || "";
	if (isValidMode(mode)) {
		const info = getNetworkInfo(mode);
		if (info) {
			if (mode === "LocalTerra") {
				// Start test on LocalTerra
				await LocalTest.startTest(new LocalTerra());
			} else {
				const terra: LCDClient = new LCDClient({
					URL: info.URL,
					chainID: info.chainID,
					gasPrices: { uusd: 0.4 },
					gasAdjustment: 1.2,
				});
				if (mode === "TestNet") {
					await TestNet.startTest(terra);
				} else if (mode === "MainNet") {
					await MainNet.startTest(terra);
				}
			}
		} else {
			console.error("Invalid network");
		}
	} else {
		console.error("Invalid network");
	}
})();