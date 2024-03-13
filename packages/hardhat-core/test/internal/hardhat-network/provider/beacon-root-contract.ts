import { assert } from "chai";

import { useFixtureProject } from "../../../helpers/project";
import { ALCHEMY_URL } from "../../../setup";
import { useEnvironment } from "../../../helpers/environment";

const BEACON_ROOT_CONTRACT_ADDRESS =
  "0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02";

describe("beacon root contract", () => {
  if (ALCHEMY_URL === undefined) {
    return;
  }

  useFixtureProject("hardhat-network-fork-after-cancun");
  useEnvironment();

  it("should get a correct response from the contract when forking", async function () {
    // timestamp of block 19427676, the block before the one that
    // is forked by this fixture project
    const timestamp = "0x65f1e46b";

    const previousBlockBeaconRoot = await this.env.network.provider.send(
      "eth_call",
      [
        {
          to: BEACON_ROOT_CONTRACT_ADDRESS,
          data: `0x${timestamp.slice(2).padStart(64, "0")}`,
        },
      ]
    );

    // see https://beaconcha.in/block/19427676
    const expectedBlockBeaconRoot =
      "0x69e7d8cbaa2aaf52eb1031d57afb6271022f422f14a4e5b9dbc8062054367c60";

    assert.equal(
      previousBlockBeaconRoot,
      expectedBlockBeaconRoot,
      "The beacon root returned by the contract is not the expected one"
    );
  });
});
