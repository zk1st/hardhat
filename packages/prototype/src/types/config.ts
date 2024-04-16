export type Primitive = number | string | boolean | bigint;

interface LazyConfigValue<T extends Primitive> {
  get(): Promise<T>;
}

// TODO: These need better names
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
}

export interface HardhatConfig {
  solidity: SolidityConfig;
}

export interface SolidityUserConfig {
  version: string;
}

export interface SolidityConfig {
  version: string;
}
