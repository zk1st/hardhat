/* eslint-disable */

import process from "process";
import { createInterface } from "node:readline";

interface UserInterruptionHooks {
  displayMessage?: (message: string) => void;

  requestInput?: (inputDescription: string) => Promise<string>;

  requestSecretInput?: (inputDescription: string) => Promise<string>;
}

const manualHooks: UserInterruptionHooks = {
  displayMessage(message: string) {
    console.log(message);
  },
  async requestInput(inputDescription: string): Promise<string> {
    if (process.stdin.isPaused()) {
      process.stdin.resume();
    }

    process.stdout.write(`${inputDescription}: `);

    const rawInputPromise = new Promise<string>((resolve, reject) => {
      let input = "";

      const onData = (data: Buffer) => {
        input += data.toString("utf-8");

        if (input.includes("\n")) {
          unsubscribe();
          resolve(input);
        }
      };

      const onEnd = () => {
        unsubscribe();
        resolve(input);
      };

      const onError = (error: Error) => {
        unsubscribe();
        reject(error);
      };

      function unsubscribe() {
        process.stdin.pause();
        process.stdin.removeListener("data", onData);
        process.stdin.removeListener("end", onEnd);
        process.stdin.removeListener("error", onError);
      }

      process.stdin.on("data", onData);
      process.stdin.on("end", onEnd);
      process.stdin.on("error", onError);
    });

    const rawInput = await rawInputPromise;

    const line = rawInput.split("\n")[0]!.trim();

    return line;
  },
  async requestSecretInput(inputDescription: string): Promise<string> {
    console.log("Oopsy, this leaks your secrets");
    return this.requestInput!(inputDescription);
  },
};

const readlineHooks: UserInterruptionHooks = {
  displayMessage(message: string) {
    console.log(message);
  },
  async requestInput(inputDescription: string): Promise<string> {
    const readline = createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    return new Promise<string>((resolve) => {
      readline.question(inputDescription + ": ", (answer) => {
        resolve(answer);
        readline.close();
      });
    });
  },
  async requestSecretInput(inputDescription: string): Promise<string> {
    const rl = createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    let echo = "*"; // or '' if you prefer
    let first: string | undefined;

    (rl as any)._writeToOutput = (c: string) => {
      if (first == undefined || c.length != 1) {
        if (first == undefined) first = c;
        if (c.startsWith(first)) {
          // rewriting prompt
          (rl as any).output?.write(first);
          c = c.slice(first.length);
        } else if (!c.trim()) {
          // user pressed enter, show the enter
          (rl as any).output?.write(c);
          c = "";
        }
      }
      for (let i = 0; i < c.length; i++) {
        // all other input, and bits after the prompt, use echo char
        (rl as any).output?.write(echo);
      }
    };

    return new Promise<string>((resolve) => {
      rl.question(inputDescription + ": ", (answer) => {
        resolve(answer);
        rl.close();
      });
    });
  },
};

const enquirerHooks: UserInterruptionHooks = {
  displayMessage(message: string) {
    console.log(message);
  },
  async requestInput(inputDescription) {
    const { default: enquirer } = await import("enquirer");
    const questions = [
      {
        type: "input",
        name: "input",
        message: inputDescription,
      },
    ];

    const answers = (await enquirer.prompt(questions)) as any;
    return answers.input;
  },
  async requestSecretInput(inputDescription) {
    const { default: enquirer } = await import("enquirer");
    const questions = [
      {
        type: "password",
        name: "input",
        message: inputDescription,
      },
    ];

    const answers = (await enquirer.prompt(questions)) as any;
    return answers.input;
  },
};


setTimeout(() => manualHooks.requestSecretInput!("Password"), 2000);


let rendering = false;

let hooksState: {
  messageToDisplay?: string;
} = {};

function render() {
  useEffect(() => {
    const hook = {}
    registerHook(...);

    return () => unregisterHook(...);
  })

  rendering = true;
}


render();
