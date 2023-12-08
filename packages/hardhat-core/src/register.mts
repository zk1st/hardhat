import debug from "debug";

import { HardhatContext } from "./internal/context.js";
import { loadConfigAndTasks } from "./internal/core/config/config-loading.js";
import { getEnvHardhatArguments } from "./internal/core/params/env-variables.js";
import { HARDHAT_PARAM_DEFINITIONS } from "./internal/core/params/hardhat-params.js";
import { Environment } from "./internal/core/runtime-environment.js";
import {
  loadTsNode,
  willRunWithTypescript,
} from "./internal/core/typescript-support.js";
import {
  disableReplWriterShowProxy,
  isNodeCalledWithoutAScript,
} from "./internal/util/console.js";

if (!HardhatContext.isCreated()) {
  // @ts-ignore
  await import("source-map-support/register.js");

  const ctx = HardhatContext.createHardhatContext();

  if (isNodeCalledWithoutAScript()) {
    disableReplWriterShowProxy();
  }

  const hardhatArguments = getEnvHardhatArguments(
    HARDHAT_PARAM_DEFINITIONS,
    process.env
  );

  if (hardhatArguments.verbose) {
    debug.enable("hardhat*");
  }

  if (willRunWithTypescript(hardhatArguments.config)) {
    await loadTsNode(
      hardhatArguments.tsconfig /* , hardhatArguments.typecheck */
    );
  }

  const { resolvedConfig, userConfig } = await loadConfigAndTasks(
    hardhatArguments
  );

  const env = new Environment(
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

  env.injectToGlobal();
}
