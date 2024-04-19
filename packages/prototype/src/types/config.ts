import { UserInterruptions } from "./user-interruptions.js";

export type PrimitiveConfigValue = number | string | boolean | bigint;

export interface LazyConfigValue<T extends PrimitiveConfigValue> {
  get(userInterruptions: UserInterruptions): Promise<T>;
}

export type HardhatUserConfigNumber = number | LazyConfigValue<number>;
export type HardhatConfigNumber = LazyConfigValue<number>;

export type HardhatUserConfigString = string | LazyConfigValue<string>;
export type HardhatConfigString = LazyConfigValue<string>;

export type HardhatUserConfigBoolean = boolean | LazyConfigValue<boolean>;
export type HardhatConfigBoolean = LazyConfigValue<boolean>;

export type HardhatUserConfigBigint = bigint | LazyConfigValue<bigint>;
export type HardhatUserBigint = LazyConfigValue<bigint>;

export interface HardhatUserConfig {
  solidity?: string | SolidityUserConfig;
  privateKey?: HardhatUserConfigString;
}

export interface HardhatConfig {
  solidity: SolidityConfig;
  privateKey: HardhatConfigString;
}

export interface SolidityUserConfig {
  version: string;
}

export interface SolidityConfig {
  version: string;
}
