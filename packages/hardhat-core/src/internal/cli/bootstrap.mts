#!/usr/bin/env node
import semver from "semver";
import chalk from "chalk";

import { SUPPORTED_NODE_VERSIONS } from "./constants.js";

if (!semver.satisfies(process.version, SUPPORTED_NODE_VERSIONS.join(" || "))) {
  console.warn(
    // this should not be needed with esModuleInterop, idk why it is
    chalk.default.yellow.bold(`WARNING:`),
    `You are currently using Node.js ${process.version}, which is not supported by Hardhat. This can lead to unexpected behavior. See https://hardhat.org/nodejs-versions`
  );
  console.log();
  console.log();
}

await import("./cli.js");
