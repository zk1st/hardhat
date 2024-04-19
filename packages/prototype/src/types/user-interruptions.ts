export interface UserInterruptions {
  displayMessage: (message: string) => Promise<void>;
  requestInput: (inputDescription: string) => Promise<string>;
  requestSecretInput: (inputDescription: string) => Promise<string>;
  // This may require a RWLock so that users don't keep printing
  // output while an interrupt is being handled. Maybe some sort of
  // `synchronized<T>(f:() => Promise<T>): Promise<T>` function.
}
