module.exports = {
  networks: {
    hardhat: {
      forking: {
        url: process.env.ALCHEMY_URL,
        blockNumber: 19427677, // a block after the cancun upgrade
      },
    },
  },
  solidity: "0.5.15",
};
