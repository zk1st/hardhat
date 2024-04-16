import builtinFunctionality from "../builtin-functionality.js";
import { HardhatPlugin } from "../types/plugins.js";
import { validatePlugin } from "./plugin-validation.js";

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

export async function getHooks<
  HookCategoryNameT extends HookCategoryName,
  HookNameT extends HookName<HookCategoryNameT>,
>(
  plugins: HardhatPlugin[],
  validatedPlugins: Set<string>,
  hookCategoryName: HookCategoryNameT,
  hookName: HookNameT,
): Promise<Array<Hook<HookCategoryNameT, HookNameT>>> {
  const maybeHooks = await Promise.all(
    plugins.map(async (plugin) =>
      getHook(plugin, validatedPlugins, hookCategoryName, hookName),
    ),
  );

  const hooks = maybeHooks.filter((h) => h !== undefined) as Array<
    Hook<HookCategoryNameT, HookNameT>
  >;

  return hooks;
}

export async function getHook<
  HookCategoryNameT extends HookCategoryName,
  HookNameT extends HookName<HookCategoryNameT>,
>(
  plugin: HardhatPlugin,
  validatedPlugins: Set<string>,
  hookCategoryName: HookCategoryNameT,
  hookName: HookNameT,
): Promise<Hook<HookCategoryNameT, HookNameT> | undefined> {
  let hookCategory = plugin.hooks[hookCategoryName];

  if (hookCategory === undefined) {
    return;
  }

  await validatePlugin(validatedPlugins, plugin);

  if (hookCategory instanceof URL) {
    const loadedHookCategory = await loadHookCategory(hookCategory);

    hookCategory = loadedHookCategory as HookCategory<HookCategoryNameT>;
  } else {
    // We don't print warning of inline hooks for the builtin functionality
    if (plugin.id !== builtinFunctionality.id) {
      console.error(
        `WARNING: Inline hooks found in plugin "${plugin.id}", category "${hookCategoryName}". User URLs in production.`,
      );
    }
  }

  const typedHookCategory = hookCategory as HookCategory<HookCategoryNameT>;
  const typedHook = typedHookCategory[hookName] as
    | Hook<HookCategoryNameT, HookNameT>
    | undefined;

  return typedHook;
}

export async function loadHookCategory(url: URL): Promise<unknown> {
  const mod = await import(url.toString());

  const obj = mod.default;

  if (obj === undefined || obj === null || Object.keys(obj).length === 0) {
    throw new Error(`Source ${url.toString()} doesn't export hooks`);
  }

  return obj;
}
