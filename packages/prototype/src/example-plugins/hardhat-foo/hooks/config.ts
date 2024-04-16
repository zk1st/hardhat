import { HardhatPluginConfigHooks } from "../../../types/plugins.js";

const hooks: HardhatPluginConfigHooks = {
  resolveUserConfig: async (userConfig, next) => {
    const resolvedConfig = await next(userConfig);
    const bar = userConfig.foo?.bar ?? [42];

    resolvedConfig.foo = {
      ...resolvedConfig.foo,
      bar: typeof bar === "number" ? [bar] : bar,
    };

    return resolvedConfig;
  },
};

export default hooks;
