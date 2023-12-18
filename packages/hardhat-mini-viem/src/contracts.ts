import hre from "hardhat";
import * as viem from "viem";
import * as chains from "viem/chains";

export const deployContract: Function = async (
  contractName: string,
  constructorArgs: any[]
) => {
  const publicClient = viem.createPublicClient({
    chain: chains.hardhat,
    transport: viem.custom(hre.network.provider),
  });

  const [defaultAccount] = await hre.network.provider.send("eth_accounts");
  const walletClient = viem.createWalletClient({
    chain: chains.hardhat,
    account: defaultAccount,
    transport: viem.custom(hre.network.provider),
  });

  const { abi, bytecode: contractBytecode } = await hre.artifacts.readArtifact(
    contractName
  );
  const hash = await walletClient.deployContract({
    abi,
    bytecode: contractBytecode as `0x${string}`,
    args: constructorArgs,
    account: defaultAccount,
  });

  const { contractAddress } = await publicClient.waitForTransactionReceipt({
    hash,
  });

  const contract = viem.getContract({
    address: contractAddress!,
    publicClient,
    walletClient,
    abi,
  });

  return contract;
};
