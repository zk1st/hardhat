import type { HardhatTask } from "hardhat/types";
import { deployTaskAction } from "./task-actions.js";

// eslint-disable-next-line import/no-default-export
export default {
  // could be called registerTasks
  defineTasks(task: HardhatTask) {
    task("accounts", "Prints the list of accounts", async (_, hre) => {
      const accounts = await hre.network.provider.send("eth_accounts");
      console.log(accounts);
    });

    task("deploy", "Deploys the specified contract")
      .addParam("contractName", "The name of the contract")
      .addOptionalVariadicPositionalParam(
        "constructorArgs",
        "Contract constructor arguments",
        []
      )
      .setAction(deployTaskAction);
  },
};
