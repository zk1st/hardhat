import hardhatFoo from "./example-plugins/hardhat-foo/index.js";
import { HardhatUserConfig } from "./types/config.js";

export default {
  plugins: [hardhatFoo],
  solidity: "0.8.22",
  foo: {
    bar: [1, 2, 3],
  },
} satisfies HardhatUserConfig;
