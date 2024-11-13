import * as Sentry from '@sentry/angular';
import { SeverityLevel } from '@sentry/angular';

export function sendSentryEvent(message: string, level: SeverityLevel = 'error'): void {
  Sentry.captureEvent({ message: message, level, tags: { type: 'webnode' } });
}

export function iOSversion(): number[] {
  if (/iP(hone|od|ad)/.test(navigator.platform)) {
    // supports iOS 2.0 and later: <http://bit.ly/TJjs1V>
    const v = (navigator.appVersion).match(/OS (\d+)_(\d+)_?(\d+)?/);
    return [parseInt(v[1], 10), parseInt(v[2], 10), parseInt(v[3] || '0', 10)];
  }
  return [0];
}
