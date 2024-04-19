import { Hooks } from "./types/hooks.js";
import { UserInterruptions as IUserInterruptions } from "./types/user-interruptions.js";

export class UserInteractionsUtils implements IUserInterruptions {
  readonly #hooks;
  constructor(hooks: Hooks) {
    this.#hooks = hooks;
  }

  public async displayMessage(message: string) {
    const hooks = await this.#hooks.getHooks(
      "userInterruption",
      "displayMessage",
    );

    let index = hooks.length - 1;
    const next = async (msg: string) => {
      if (index >= 0) {
        return hooks[index--]!(msg, next);
      }

      throw new Error(
        `No hook handled the displayMessage user interruption with message: ${message}`,
      );
    };

    return next(message);
  }

  public async requestInput(inputDescription: string): Promise<string> {
    const hooks = await this.#hooks.getHooks(
      "userInterruption",
      "requestInput",
    );

    let index = hooks.length - 1;
    const next = async (id: string) => {
      if (index >= 0) {
        return hooks[index--]!(id, next);
      }

      throw new Error(
        `No hook handled the requestInput user interruption for: ${inputDescription}`,
      );
    };

    return next(inputDescription);
  }
  public async requestSecretInput(inputDescription: string): Promise<string> {
    const hooks = await this.#hooks.getHooks(
      "userInterruption",
      "requestSecretInput",
    );

    let index = hooks.length - 1;
    const next = async (id: string) => {
      if (index >= 0) {
        return hooks[index--]!(id, next);
      }

      throw new Error(
        `No hook handled the requestSecretInput user interruption for: ${inputDescription}`,
      );
    };

    return next(inputDescription);
  }
}
