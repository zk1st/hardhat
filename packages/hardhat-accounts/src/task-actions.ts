import { ActionType } from "hardhat/types";

export const deployTaskAction: ActionType<any> = async ({
  contractName,
  constructorArgs,
}) => {
  const { deployContract } = await import(
    "@nomicfoundation/hardhat-mini-viem/contracts"
  );
  const contract = await deployContract(contractName, constructorArgs);
  console.log(`Contract deployed to: ${contract.address}`);
};
