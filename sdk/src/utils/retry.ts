export async function withRetry<T>(
  fn: () => Promise<T>,
  attempts: number = 3,
  delayMs: number = 500,
): Promise<T> {
  let lastError: unknown;
  for (let i = 0; i < attempts; i++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
      if (i < attempts - 1) await sleep(delayMs * 2 ** i);
    }
  }
  throw lastError;
}

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));
