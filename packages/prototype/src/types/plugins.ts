import { HardhatConfig, HardhatUserConfig } from "./config.js";

// We add the plugins to the config type here
// to avoid a circular dependency and/or having
// a huge file with everything.
declare module "./config.js" {
  export interface HardhatUserConfig {
    plugins?: HardhatPlugin[];
  }

  export interface HardhatConfig {
    // The plugins in a topological order
    plugins: HardhatPlugin[];
  }
}

export interface HardhatPlugin {
  id: string;
  npmPackage?: string;
  hooks: HardhatPluginHooks;
  dependencies?: HardhatPlugin[];
}

export interface HardhatPluginHooks {
  config?: HardhatPluginConfigHooks | URL;
}

export type ExtensionHook<ValueT> = (
  value: ValueT,
  next: (v: ValueT) => Promise<ValueT>,
) => Promise<ValueT>;

export type ExtendUserConfigHook = ExtensionHook<HardhatUserConfig>;

export interface HardhatPluginConfigHooks {
  extendUserConfig?: ExtendUserConfigHook;
  validateUserConfig?: (
    config: HardhatUserConfig,
  ) => Promise<HardhatUserConfigValidationError[]>;
  resolveUserConfig?: (
    config: HardhatUserConfig,
    next: (userConfig: HardhatUserConfig) => Promise<HardhatConfig>,
  ) => Promise<HardhatConfig>;
}

export interface HardhatUserConfigValidationError {
  path: Array<string | number>;
  message: string;
}
