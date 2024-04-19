export interface UserInterruptions {
  displayMessage: (
    message: string,
    requester: string,
    defaultHandler?: () => Promise<void>,
  ) => Promise<void>;

  requestInput: (
    inputDescription: string,
    requester: string,
    defaultHandler?: () => Promise<string>,
  ) => Promise<string>;

  requestSecretInput: (
    inputDescription: string,
    requester: string,
    defaultHandler?: () => Promise<string>,
  ) => Promise<string>;

  uninterrupted<ReturnT>(f: () => ReturnT): Promise<Awaited<ReturnT>>;
}
