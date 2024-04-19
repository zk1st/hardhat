import type { HardhatRuntimeEnvionment as IHardhatRuntimeEnvionment } from "./types/hre.js";
import { HardhatUserConfig, HardhatConfig } from "./types/config.js";
import { Hooks } from "./types/hooks.js";
import { HooksUtils } from "./hook-utils.js";
import builtinFunctionality from "./builtin-functionality.js";
import { reverseTopologicalSort } from "./plugins/sort.js";
import {
  HardhatPlugin,
  HardhatUserConfigValidationError,
} from "./types/plugins.js";
import { UserInterruptions } from "./types/user-interruptions.js";
import { UserInteractionsUtils } from "./user-interruptions.js";

export class HardhatRuntimeEnvironment implements IHardhatRuntimeEnvionment {
  public static async create(
    config: HardhatUserConfig,
  ): Promise<HardhatRuntimeEnvironment> {
    // Clone with lodash or https://github.com/davidmarkclements/rfdc
    const clonedConfig = config;

    // Topological sort of plugins
    const sortedPlugins = reverseTopologicalSort([
      builtinFunctionality,
      ...(clonedConfig.plugins ?? []),
    ]);

    const hooks = new HooksUtils(sortedPlugins);
    const interruptions = new UserInteractionsUtils(hooks);

    // extend user config:
    const userConfig = await runUserConfigExtensions(hooks, clonedConfig);

    // validate config
    const userConfigValidationErrors = await validateUserConfig(
      hooks,
      userConfig,
    );

    if (userConfigValidationErrors.length > 0) {
      throw new Error(
        `Invalid config:\n\t${userConfigValidationErrors
          .map(
            (error) =>
              `* Config error in .${error.path.join(".")}: ${error.message}`,
          )
          .join("\n\t")}`,
      );
    }

    // Resolve config

    const resolvedConfig = await resolveUserConfig(
      hooks,
      sortedPlugins,
      config,
    );

    return new HardhatRuntimeEnvironment(
      userConfig,
      resolvedConfig,
      hooks,
      interruptions,
    );
  }

  private constructor(
    public readonly userConfig: HardhatUserConfig,
    public readonly config: HardhatConfig,
    public readonly hooks: Hooks,
    public readonly interruptions: UserInterruptions,
  ) {}
}

async function runUserConfigExtensions(
  hooks: Hooks,
  config: HardhatUserConfig,
): Promise<HardhatUserConfig> {
  const extendUserConfigHooks = await hooks.getHooks(
    "config",
    "extendUserConfig",
  );

  let index = extendUserConfigHooks.length - 1;
  const next = async (userConfig: HardhatUserConfig) => {
    if (index >= 0) {
      return extendUserConfigHooks[index--]!(userConfig, next);
    }

    return userConfig;
  };

  return next(config);
}

async function validateUserConfig(
  hooks: Hooks,
  config: HardhatUserConfig,
): Promise<HardhatUserConfigValidationError[]> {
  const validateUserConfigHooks = await hooks.getHooks(
    "config",
    "validateUserConfig",
  );

  const hookResults = await Promise.all(
    validateUserConfigHooks.map(async (h) => h(config)),
  );

  return hookResults.flat(1);
}

async function resolveUserConfig(
  hooks: Hooks,
  sortedPlugins: HardhatPlugin[],
  config: HardhatUserConfig,
): Promise<HardhatConfig> {
  const initialResolvedConfig = {
    plugins: sortedPlugins,
  } as HardhatConfig;

  const resolveUserConfigHooks = await hooks.getHooks(
    "config",
    "resolveUserConfig",
  );

  let index = resolveUserConfigHooks.length - 1;
  const next = async (userConfig: HardhatUserConfig) => {
    if (index >= 0) {
      return resolveUserConfigHooks[index--]!(userConfig, next);
    }

    return initialResolvedConfig;
  };

  return next(config);
}
