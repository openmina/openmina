import { platformBrowserDynamic } from '@angular/platform-browser-dynamic';

import { AppModule } from './app/app.module';
import { CONFIG } from '@shared/constants/config';
import * as Sentry from '@sentry/angular';
import type { ErrorEvent } from '@sentry/types/build/types/event';

if (CONFIG.production) {
  initSentry();
}

platformBrowserDynamic().bootstrapModule(AppModule)
  .catch(err => console.error(err));

function initSentry(): void {
  if (CONFIG.sentry) {
    const clientFingerprint = (Math.random() * 1e9).toString();
    Sentry.init({
      dsn: CONFIG.sentry.dsn,
      integrations: [
        Sentry.browserTracingIntegration(),
        Sentry.replayIntegration(),
      ],
      tracesSampleRate: 1.0,
      profilesSampleRate: 1.0,
      tracePropagationTargets: [...CONFIG.sentry?.tracingOrigins, ...CONFIG.configs.map((config) => config.url).filter(Boolean)],
      replaysSessionSampleRate: 1.0,
      replaysOnErrorSampleRate: 0.1,
      beforeSend: (event: ErrorEvent) => {
        event.fingerprint = [clientFingerprint];
        return event;
      },
    });
  }
}
