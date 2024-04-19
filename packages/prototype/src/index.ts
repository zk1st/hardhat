import path, { isAbsolute } from "path";
import { createInterface } from "readline";
import { HardhatRuntimeEnvironment } from "./hre.js";
import { UserInterruptions } from "./types/user-interruptions.js";
import { UserInterruptionsHooks } from "./types/plugins.js";

await main();

async function main() {
  const then = process.hrtime.bigint();
  const [_node, _main, configPath] = process.argv;

  if (configPath === undefined) {
    console.error("No config file provided");
    return;
  }

  const resolvedConfigPath = isAbsolute(configPath)
    ? configPath
    : path.join(process.cwd(), configPath);

  const config = (await import(resolvedConfigPath)).default;

  if (config === undefined) {
    console.error("No config returned");
    return;
  }

  const hre = await HardhatRuntimeEnvironment.create(config);

  const now = process.hrtime.bigint();
  console.log("Time to initialize the HRE (ms):", (now - then) / 1000000n);

  console.log("\n\n\n");

  await ignition(hre);
}

async function ignition(hre: HardhatRuntimeEnvironment) {
  const ignitionInterruptionHooks: UserInterruptionsHooks = {
    async requestSecretInput(
      inputDescription: string,
      next: (m: string) => Promise<string>,
    ): Promise<string> {
      console.log("Ignition request secret input");
      return readlineRequestSecretInput(inputDescription, next);
    },
  };

  hre.hooks.registerHooks("userInterruption", ignitionInterruptionHooks);

  try {
    // We print something complex here
    for (let i = 0; i < 20; i++) {
      console.log(i);

      if (i === 10) {
        // Sudendly we get an interruption, which is printed with our own function
        const pk = await hre.config.privateKey.get(hre.interruptions);
        await hre.interruptions.displayMessage(`Got private key: ${pk}`);
      }

      await new Promise((resolve) => setTimeout(resolve, 300));
    }
  } finally {
    hre.hooks.unregisterHooks("userInterruption", ignitionInterruptionHooks);
  }

  const pk2 = await hre.config.privateKey.get(hre.interruptions);
  await hre.interruptions.displayMessage(`Got private key: ${pk2}`);
}

async function readlineRequestSecretInput(
  inputDescription: string,
  _next: (m: string) => Promise<string>,
): Promise<string> {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  const echo = "*"; // or '' if you prefer
  let first: string | undefined;

  (rl as any)._writeToOutput = (c: string) => {
    if (first === undefined || c.length !== 1) {
      if (first === undefined) first = c;
      if (c.startsWith(first)) {
        // rewriting prompt
        (rl as any).output?.write(first);
        c = c.slice(first.length);
      } else if (c.trim() === "") {
        // user pressed enter, show the enter
        (rl as any).output?.write(c);
        c = "";
      }
    }
    for (const _ of c) {
      // all other input, and bits after the prompt, use echo char
      (rl as any).output?.write(echo);
    }
  };

  return new Promise<string>((resolve) => {
    rl.question(`${inputDescription}: `, (answer) => {
      resolve(answer);
      rl.close();
    });
  });
}
