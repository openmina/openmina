import * as Sentry from '@sentry/angular';

export function sendSentryEvent(message: string): void {
  Sentry.captureEvent({ message: message, tags: { type: 'webnode' } });
}
