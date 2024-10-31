import { platformBrowserDynamic } from '@angular/platform-browser-dynamic';

import { AppModule } from '@app/app.module';
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
    Sentry.init({
      dsn: CONFIG.sentry.dsn,
      integrations: [
        Sentry.browserTracingIntegration(),
        Sentry.replayIntegration(),
      ],
      tracesSampleRate: 1.0,
      tracePropagationTargets: [...CONFIG.sentry?.tracingOrigins, ...CONFIG.configs.map((config) => config.url).filter(Boolean)],
      replaysSessionSampleRate: 1.0,
      replaysOnErrorSampleRate: 0.1,
      beforeSend: (event: ErrorEvent) => {
        event.fingerprint = [(Math.random() * 10000000).toString()];
        return event;
      },
    });
  }
}
