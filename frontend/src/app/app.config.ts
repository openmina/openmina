import {
  ApplicationConfig,
  ErrorHandler,
  importProvidersFrom,
  Injectable,
  LOCALE_ID,
} from '@angular/core';
import { provideRouter, Router } from '@angular/router';
import { provideAnimations } from '@angular/platform-browser/animations';
import {
  provideHttpClient,
  withInterceptorsFromDi,
} from '@angular/common/http';
import {
  provideClientHydration,
  withIncrementalHydration,
} from '@angular/platform-browser';
import { provideStore } from '@ngrx/store';
import { EffectsModule, provideEffects } from '@ngrx/effects';
import {
  provideRouterStore,
  routerReducer,
  RouterStateSerializer,
} from '@ngrx/router-store';
import { provideStoreDevtools } from '@ngrx/store-devtools';
import * as Sentry from '@sentry/angular';
import { registerLocaleData } from '@angular/common';
import {
  GlobalErrorHandlerService,
  MergedRouterStateSerializer,
  safelyExecuteInBrowser,
  THEME_PROVIDER,
} from '@openmina/shared';
import { CONFIG } from '@shared/constants/config';
import localeFr from '@angular/common/locales/fr';
import localeEn from '@angular/common/locales/en';
import { metaReducers, reducers } from '@app/app.setup';
import { AppEffects } from '@app/app.effects';
import { generateRoutes } from '@app/app.routing';

registerLocaleData(localeFr, 'fr');
registerLocaleData(localeEn, 'en');

@Injectable()
export class AppGlobalErrorhandler implements ErrorHandler {
  constructor(private errorHandlerService: GlobalErrorHandlerService) {
    safelyExecuteInBrowser(() => {
      this.setupErrorHandlers();
    });

    if (WebAssembly) {
      this.interceptWebAssembly();
    }
  }

  private setupErrorHandlers(): void {
    const self = this;

    // Global error handler
    window.onerror = function (msg, url, line, column, error) {
      self.handleError(error || msg);
      return false;
    };

    // Unhandled promise rejections
    window.onunhandledrejection = function (event) {
      event.preventDefault();
      self.handleError(event.reason);
    };

    // Regular error listener
    window.addEventListener(
      'error',
      (event: ErrorEvent) => {
        event.preventDefault();
        this.handleError(event.error);
      },
      { capture: true },
    );

    // Override console.error with proper error extraction
    const originalConsoleError = console.error;
    console.error = (...args) => {
      // Find the actual error object in the arguments
      const error = args.find(arg => arg instanceof Error) || args.join(' ');

      this.handleError(error);
      originalConsoleError.apply(console, args);
    };
  }

  private interceptWebAssembly(): void {
    const self = this;

    const originalInstantiateStreaming = WebAssembly.instantiateStreaming;
    if (originalInstantiateStreaming) {
      WebAssembly.instantiateStreaming = async function (
        response: any,
        importObject?: any,
      ): Promise<any> {
        try {
          return await originalInstantiateStreaming.call(
            WebAssembly,
            response,
            importObject,
          );
        } catch (error) {
          self.handleError(error);
          throw error;
        }
      };
    }

    const originalInstantiate = WebAssembly.instantiate;
    WebAssembly.instantiate = async function (
      moduleObject: any,
      importObject?: any,
    ): Promise<any> {
      try {
        return await originalInstantiate.call(
          WebAssembly,
          moduleObject,
          importObject,
        );
      } catch (error) {
        self.handleError(error);
        throw error;
      }
    };
  }

  handleError(error: any): void {
    Sentry.captureException(error);
    if (typeof error === 'string') {
      error = new Error(error);
    }
    this.errorHandlerService.handleError(error);
  }
}

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(generateRoutes()),
    provideAnimations(),
    provideClientHydration(withIncrementalHydration()),
    provideHttpClient(withInterceptorsFromDi()),
    provideStore(
      {
        ...reducers,
        router: routerReducer,
      } as any,
      {
        metaReducers,
        runtimeChecks: {
          strictStateImmutability: true,
          strictActionImmutability: true,
          strictActionWithinNgZone: true,
          strictStateSerializability: true,
        },
      },
    ),
    provideRouterStore({ stateKey: 'router' }),
    {
      provide: RouterStateSerializer,
      useClass: MergedRouterStateSerializer,
    },
    provideEffects(AppEffects),
    !CONFIG.production
      ? provideStoreDevtools({ maxAge: 150, connectInZone: true })
      : [],
    importProvidersFrom(EffectsModule.forRoot()),
    // Your custom providers
    THEME_PROVIDER,
    { provide: LOCALE_ID, useValue: 'en' },
    { provide: ErrorHandler, useValue: Sentry.createErrorHandler() },
    {
      provide: ErrorHandler,
      useClass: AppGlobalErrorhandler,
      deps: [GlobalErrorHandlerService],
      multi: false,
    },
    { provide: Sentry.TraceService, deps: [Router] },
  ],
};
