import { HardhatPlugin } from "./plugins.js";

export type HookCategoryName = keyof HardhatPlugin["hooks"];

export type HookCategory<HookCategoryNameT extends HookCategoryName> = Exclude<
  HardhatPlugin["hooks"][HookCategoryNameT],
  undefined | URL
>;

export type HookName<HookCategoryNameT extends HookCategoryName> =
  keyof HookCategory<HookCategoryNameT>;

export type Hook<
  HookCategoryNameT extends HookCategoryName,
  HookNameT extends HookName<HookCategoryNameT>,
> = Exclude<HookCategory<HookCategoryNameT>[HookNameT], undefined>;

export interface Hooks {
  getHooks<
    HookCategoryNameT extends HookCategoryName,
    HookNameT extends HookName<HookCategoryNameT>,
  >(
    hookCategoryName: HookCategoryNameT,
    hookName: HookNameT,
  ): Promise<Array<Hook<HookCategoryNameT, HookNameT>>>;

  registerHooks<HookCategoryNameT extends HookCategoryName>(
    hookCategoryName: HookCategoryNameT,
    hookCategory: HookCategory<HookCategoryNameT>,
  ): void;

  unregisterHooks<HookCategoryNameT extends HookCategoryName>(
    hookCategoryName: HookCategoryNameT,
    hookCategory: HookCategory<HookCategoryNameT>,
  ): void;
}
