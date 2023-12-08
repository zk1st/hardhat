import debug from "debug";

import { HardhatRuntimeEnvironment } from "../../types/index.js";
import { HardhatContext } from "../context.js";
import { loadConfigAndTasks } from "../core/config/config-loading.js";
import { HardhatError } from "../core/errors.js";
import { ERRORS } from "../core/errors-list.js";
import { getEnvHardhatArguments } from "../core/params/env-variables.js";
import { HARDHAT_PARAM_DEFINITIONS } from "../core/params/hardhat-params.js";
import { Environment } from "../core/runtime-environment.js";

let ctx: HardhatContext;
let env: HardhatRuntimeEnvironment;

if (HardhatContext.isCreated()) {
  ctx = HardhatContext.getHardhatContext();

  // The most probable reason for this to happen is that this file was imported
  // from the config file
  if (ctx.environment === undefined) {
    throw new HardhatError(ERRORS.GENERAL.LIB_IMPORTED_FROM_THE_CONFIG);
  }

  env = ctx.environment;
} else {
  ctx = HardhatContext.createHardhatContext();

  const hardhatArguments = getEnvHardhatArguments(
    HARDHAT_PARAM_DEFINITIONS,
    process.env
  );

  if (hardhatArguments.verbose) {
    debug.enable("hardhat*");
  }

  const { resolvedConfig, userConfig } = await loadConfigAndTasks(
    hardhatArguments
  );

  env = new Environment(
    resolvedConfig,
    hardhatArguments,
    ctx.tasksDSL.getTaskDefinitions(),
    ctx.tasksDSL.getScopesDefinitions(),
    ctx.environmentExtenders,
    ctx.experimentalHardhatNetworkMessageTraceHooks,
    userConfig,
    ctx.providerExtenders
  );

  ctx.setHardhatRuntimeEnvironment(env);
}

// eslint-disable-next-line import/no-default-export
export default env;
