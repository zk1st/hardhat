declare module "../../types/config.js" {
  interface FooUserConfig {
    bar?: number | number[];
  }

  interface FooConfig {
    bar: number[];
  }

  interface HardhatUserConfig {
    foo?: FooUserConfig;
  }

  interface HardhatConfig {
    foo: FooConfig;
  }
}
