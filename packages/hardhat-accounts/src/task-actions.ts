import { ActionType } from "hardhat/types";
import { deployContract } from "@nomicfoundation/hardhat-mini-viem/contracts";

export const deployTaskAction: ActionType<any> = async ({
  contractName,
  constructorArgs,
}) => {
  const contract = await deployContract(contractName, constructorArgs);
  console.log(`Contract deployed to: ${contract.address}`);
};
