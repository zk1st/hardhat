import { HardhatPlugin } from "../types/plugins.js";

export async function validatePlugin(
  validatedPlugins: Set<string>,
  plugin: HardhatPlugin,
) {
  if (validatedPlugins.has(plugin.id)) {
    return;
  }

  // If it has an npm package, validate their peer dependencies
  // If any is missing, throw

  validatedPlugins.add(plugin.id);
}
