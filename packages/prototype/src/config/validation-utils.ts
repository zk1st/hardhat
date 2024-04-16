import { ZodType, ZodTypeDef, ZodIssue, z } from "zod";
import { HardhatUserConfigValidationError } from "../types/plugins.js";
import { HardhatUserConfig } from "../types/config.js";

export const LazyConfigValue = z.object({
  get: z.function(),
});

export const HardhatUserConfigNumberType = z.union([
  z.number(),
  LazyConfigValue,
]);
export const HardhatUserConfigStringType = z.union([
  z.string(),
  LazyConfigValue,
]);
export const HardhatUserConfigBooleanType = z.union([
  z.boolean(),
  LazyConfigValue,
]);
export const HardhatUserConfigBigintType = z.union([
  z.bigint(),
  LazyConfigValue,
]);

export async function validateUserConfig<
  Output,
  Def extends ZodTypeDef = ZodTypeDef,
  Input = Output,
>(
  config: HardhatUserConfig,
  configType: ZodType<Output, Def, Input>,
): Promise<HardhatUserConfigValidationError[]> {
  const result = await configType.safeParseAsync(config);

  if (result.success) {
    return [];
  } else {
    return result.error.errors.map((issue) =>
      zodIssueToValidationError(config, configType, issue),
    );
  }
}

export function zodIssueToValidationError<
  Output,
  Def extends ZodTypeDef = ZodTypeDef,
  Input = Output,
>(
  _config: HardhatUserConfig,
  _configType: ZodType<Output, Def, Input>,
  zodIssue: ZodIssue,
): HardhatUserConfigValidationError {
  if (zodIssue.code === "invalid_union") {
    return {
      path: zodIssue.path,
      message: `Expected ${zodIssue.unionErrors
        .flatMap((ue) => ue.errors)
        .map((zi) => {
          if (zi.code === "invalid_type") {
            return zi.expected;
          }

          return "(please see the docs)";
        })
        .join(" or ")}`,
    };
  }

  return { path: zodIssue.path, message: zodIssue.message };
}
