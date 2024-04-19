import hardhatFoo from "./example-plugins/hardhat-foo/index.js";
import { HardhatUserConfig } from "./types/config.js";

export default {
  plugins: [hardhatFoo],
  solidity: "0.8.22",
  foo: {
    bar: 12,
  },
  privateKey: {
    get: async (interruptions) => {
      return interruptions.requestSecretInput("Private key");
    },
  },
} satisfies HardhatUserConfig;
