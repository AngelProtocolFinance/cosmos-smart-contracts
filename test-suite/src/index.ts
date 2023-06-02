import * as readline from "readline/promises";
import { TestsUniverse } from "./environments";

//----------------------------------------------------------------------------------------
// Test-suite for Local, TestNet, and MainNet
//----------------------------------------------------------------------------------------
(async () => {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  const networkOpts: string[] = Object.keys(TestsUniverse);
  const network = await rl
    .question(`Network? ${networkOpts} `)
    .then((input) => input.toLowerCase());
  if (networkOpts.indexOf(network) == -1) {
    console.error(
      "Invalid Network input! Valid network choices are:\n",
      networkOpts
    );
    process.exit(0);
  }

  const actionOpts: string[] = Object.keys(TestsUniverse[network]);
  const action = await rl
    .question(`Action to take? ${actionOpts} `)
    .then((input) => input.toLowerCase());
  if (actionOpts.indexOf(action) == -1) {
    console.error(
      `Invalid Action invoked for given network(${network})! Valid actions are:\n`,
      actionOpts
    );
    process.exit(0);
  }

  // check if we need to find the sub-aciton target (not needed for tests & any localibc items)
  if (network != "localibc" && action.toLowerCase() != "tests") {
    const targetOpts: string[] = Object.keys(
      TestsUniverse[network][action]
    );
    const target = await rl
      .question(`Target? ${targetOpts} `)
      .then((input) => input.toLowerCase());
    if (targetOpts.indexOf(target) == -1) {
      console.error(
        `Invalid Target invoked for given Network(${network}) and Action(${action})! Valid targets are:\n`,
        targetOpts
      );
      process.exit(0);
    }
    rl.close();
    await TestsUniverse[network][action][target]();
  }
  rl.close();
  await TestsUniverse[network][action]();
})();
