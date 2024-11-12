import * as Sentry from '@sentry/angular';
import { SeverityLevel } from '@sentry/angular';

export function sendSentryEvent(message: string, level: SeverityLevel = 'error'): void {
  Sentry.captureEvent({ message: message, level, tags: { type: 'webnode' } });
}
