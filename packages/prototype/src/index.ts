import path, { isAbsolute } from "path";
import { getHooks } from "./plugins/hook-utils.js";
import { HardhatConfig, HardhatUserConfig } from "./types/config.js";

import {
  HardhatPlugin,
  HardhatUserConfigValidationError,
} from "./types/plugins.js";
import { reverseTopologicalSort } from "./plugins/sort.js";
import builtinFunctionality from "./builtin-functionality.js";

await main();

async function main() {
  const then = process.hrtime.bigint();
  const [_node, _main, configPath] = process.argv;

  if (configPath === undefined) {
    console.error("No config file provided");
    return;
  }

  const resolvedConfigPath = isAbsolute(configPath)
    ? configPath
    : path.join(process.cwd(), configPath);

  const config = (await import(resolvedConfigPath)).default;

  if (config === undefined) {
    console.error("No config returned");
    return;
  }

  const hre = await createHardhatRuntimeEnvionment(config);

  const now = process.hrtime.bigint();
  console.log("Time to initialize the HRE (ms):", (now - then) / 1000000n);

  console.log(hre.config);
}

async function createHardhatRuntimeEnvionment(config: HardhatUserConfig) {
  // Clone with lodash or https://github.com/davidmarkclements/rfdc
  const clonedConfig = config;

  // Topological sort of plugins
  const sortedPlugins = reverseTopologicalSort([
    builtinFunctionality,
    ...(clonedConfig.plugins ?? []),
  ]);

  // Validated plugins set to avoid re-validations
  const validatedPlugins: Set<string> = new Set();

  // extend user config:
  const userConfig = await runUserConfigExtensions(
    sortedPlugins,
    validatedPlugins,
    clonedConfig,
  );

  // validate config
  const userConfigValidationErrors = await validateUserConfig(
    sortedPlugins,
    validatedPlugins,
    userConfig,
  );

  if (userConfigValidationErrors.length > 0) {
    throw new Error(
      `Invalid config:\n\t${userConfigValidationErrors
        .map((error) => `Config error in ${error.path}: ${error.message}`)
        .join("\n\t")}`,
    );
  }

  // Resolve config

  const resolvedConfig = await resolveUserConfig(
    sortedPlugins,
    validatedPlugins,
    config,
  );

  return {
    config: resolvedConfig,
    userConfig, // TODO: Why do we use it?
    plugins: {
      validatedPlugins,
    },
    // Network
    // Build system
    // Task runner
  };
}

async function runUserConfigExtensions(
  sortedPlugins: HardhatPlugin[],
  validatedPlugins: Set<string>,
  config: HardhatUserConfig,
): Promise<HardhatUserConfig> {
  const hooks = await getHooks(
    sortedPlugins,
    validatedPlugins,
    "config",
    "extendUserConfig",
  );

  let index = hooks.length - 1;
  const next = async (userConfig: HardhatUserConfig) => {
    if (index >= 0) {
      return hooks[index--]!(userConfig, next);
    }

    return userConfig;
  };

  return next(config);
}

async function validateUserConfig(
  sortedPlugins: HardhatPlugin[],
  validatedPlugins: Set<string>,
  config: HardhatUserConfig,
): Promise<HardhatUserConfigValidationError[]> {
  const hooks = await getHooks(
    sortedPlugins,
    validatedPlugins,
    "config",
    "validateUserConfig",
  );

  const hookResults = await Promise.all(hooks.map(async (h) => h(config)));

  return hookResults.flat(1);
}

async function resolveUserConfig(
  sortedPlugins: HardhatPlugin[],
  validatedPlugins: Set<string>,
  config: HardhatUserConfig,
): Promise<HardhatConfig> {
  const initialResolvedConfig = {
    plugins: sortedPlugins,
  } as HardhatConfig;

  const hooks = await getHooks(
    sortedPlugins,
    validatedPlugins,
    "config",
    "resolveUserConfig",
  );

  let index = hooks.length - 1;
  const next = async (userConfig: HardhatUserConfig) => {
    if (index >= 0) {
      return hooks[index--]!(userConfig, next);
    }

    return initialResolvedConfig;
  };

  return next(config);
}
