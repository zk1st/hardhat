import { Hooks } from "./types/hooks.js";
import { UserInterruptions as IUserInterruptions } from "./types/user-interruptions.js";

export class UserInteractionsUtils implements IUserInterruptions {
  readonly #hooks;
  readonly #mutex = new AsyncMutex();

  constructor(hooks: Hooks) {
    this.#hooks = hooks;
  }

  public async displayMessage(
    message: string,
    requester: string,
    defaultHandler?: () => Promise<void>,
  ): Promise<void> {
    return this.#mutex.excluiveRun(async () => {
      const hooks = await this.#hooks.getHooks(
        "userInterruption",
        "displayMessage",
      );

      let index = hooks.length - 1;
      const next = async (msg: string, r: string) => {
        if (index >= 0) {
          return hooks[index--]!(msg, r, next);
        }

        if (defaultHandler !== undefined) {
          return defaultHandler();
        } else {
          return fallbackHandler.displayMessage(msg, r);
        }
      };

      return next(message, requester);
    });
  }

  public async requestInput(
    inputDescription: string,
    requester: string,
    defaultHandler?: () => Promise<string>,
  ): Promise<string> {
    return this.#mutex.excluiveRun(async () => {
      const hooks = await this.#hooks.getHooks(
        "userInterruption",
        "requestInput",
      );

      let index = hooks.length - 1;
      const next = async (id: string, r: string) => {
        if (index >= 0) {
          return hooks[index--]!(id, r, next);
        }

        if (defaultHandler !== undefined) {
          return defaultHandler();
        } else {
          return fallbackHandler.requestInput(id, r);
        }
      };

      return next(inputDescription, requester);
    });
  }

  public async requestSecretInput(
    inputDescription: string,
    requester: string,
    defaultHandler?: () => Promise<string>,
  ): Promise<string> {
    return this.#mutex.excluiveRun(async () => {
      const hooks = await this.#hooks.getHooks(
        "userInterruption",
        "requestSecretInput",
      );

      let index = hooks.length - 1;
      const next = async (id: string, r: string) => {
        if (index >= 0) {
          return hooks[index--]!(id, r, next);
        }

        if (defaultHandler !== undefined) {
          return defaultHandler();
        } else {
          return fallbackHandler.requestInput(id, r);
        }
      };

      return next(inputDescription, requester);
    });
  }

  public async uninterrupted<ReturnT>(
    f: () => ReturnT,
  ): Promise<Awaited<ReturnT>> {
    return this.#mutex.excluiveRun(f);
  }
}

const fallbackHandler = {
  async displayMessage(message: string, requester: string) {
    console.log(`[${requester}]: ${message}`);
  },
  async requestInput(inputDescription: string, requester: string) {
    const { default: enquirer } = await import("enquirer");
    const questions = [
      {
        type: "input",
        name: "input",
        message: `[${requester}] ${inputDescription}`,
      },
    ];

    const answers = (await enquirer.prompt(questions)) as any;
    return answers.input;
  },
  async requestSecretInput(inputDescription: string, requester: string) {
    const { default: enquirer } = await import("enquirer");
    const questions = [
      {
        type: "password",
        name: "input",
        message: `[${requester}] ${inputDescription}`,
      },
    ];

    const answers = (await enquirer.prompt(questions)) as any;
    return answers.input;
  },
};

class AsyncMutex {
  #acquired = false;
  #queue: Array<() => void> = [];

  /**
   * Aquires the mutex, running the provided function exclusively,
   * and releasing it afterwards.
   *
   * @param f The function to run.
   * @returns The result of the function.
   */
  public async excluiveRun<ReturnT>(
    f: () => ReturnT,
  ): Promise<Awaited<ReturnT>> {
    const release = await this._acquire();

    try {
      // eslint-disable-next-line @typescript-eslint/return-await, @typescript-eslint/await-thenable
      return await f();
    } finally {
      await release();
    }
  }

  /**
   * Aquires the mutex, returning a function that releases it.
   */
  private async _acquire(): Promise<() => Promise<void>> {
    if (!this.#acquired) {
      this.#acquired = true;
      return async () => {
        this.#acquired = false;
        const next = this.#queue.shift();
        if (next !== undefined) {
          next();
        }
      };
    }

    return new Promise<() => Promise<void>>((resolve) => {
      this.#queue.push(() => {
        resolve(this._acquire());
      });
    });
  }
}
