import type { HardhatPlugin } from "./types/plugins.js";
import { z } from "zod";
import {
  HardhatUserConfigStringType,
  validateUserConfigZodType,
} from "./config/validation-utils.js";
import { LazyConfigValue, PrimitiveConfigValue } from "./types/config.js";
import { UserInterruptions } from "./types/user-interruptions.js";

const SolidityUserConfig = z.object({
  version: z.string(),
});

const HardhatUserConfig = z.object({
  solidity: z.optional(z.union([z.string(), SolidityUserConfig])),
  privateKey: z.optional(HardhatUserConfigStringType),
});

class NotLazyConfigValue<T extends PrimitiveConfigValue>
  implements LazyConfigValue<T>
{
  constructor(private readonly _value: T) {}

  public async get(_: UserInterruptions) {
    return this._value;
  }
}

export default {
  id: "builtin-functionality",
  hooks: {
    config: {
      validateUserConfig: async (config) => {
        return validateUserConfigZodType(config, HardhatUserConfig);
      },
      resolveUserConfig: async (userConfig, next) => {
        const resolvedConfig = await next(userConfig);

        const version =
          typeof userConfig.solidity === "string"
            ? userConfig.solidity
            : userConfig.solidity?.version ?? "0.8.2";

        resolvedConfig.solidity = {
          ...resolvedConfig.solidity,
          version,
        };

        const privateKey = userConfig.privateKey ?? "0xnope";

        if (typeof privateKey === "string") {
          resolvedConfig.privateKey = new NotLazyConfigValue(privateKey);
        } else {
          resolvedConfig.privateKey = privateKey;
        }

        return resolvedConfig;
      },
    },
  },
  dependencies: [],
} satisfies HardhatPlugin;
