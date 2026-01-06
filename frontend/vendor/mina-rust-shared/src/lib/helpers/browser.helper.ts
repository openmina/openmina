export function isBrowser(): boolean {
  return typeof window !== 'undefined' && typeof window.document !== 'undefined';
}

export function getOrigin(): string {
  if (isBrowser()) {
    return window.location.origin;
  }
  return '';
}

export function safelyExecuteInBrowser<T>(fn: () => T, fallback?: T): void {
  if (isBrowser()) {
    try {
      fn();
    } catch (error) {
      console.error('Error executing browser function:', error);
    }
  }
}

export function getWindow(): Window | null {
  if (isBrowser()) {
    return window;
  }
  return null;
}

export function getLocalStorage(): Storage | null {
  if (isBrowser()) {
    return window.localStorage;
  }
  return null;
}
