import { APP_INITIALIZER, ErrorHandler, Injectable, LOCALE_ID, NgModule } from '@angular/core';
import { BrowserModule, provideClientHydration } from '@angular/platform-browser';

import { AppComponent } from './app.component';
import { AppRouting } from './app.routing';
import { MatSidenavModule } from '@angular/material/sidenav';
import {
  CopyComponent,
  GlobalErrorHandlerService,
  HorizontalMenuComponent,
  NgrxRouterStoreModule,
  OpenminaEagerSharedModule,
  safelyExecuteInBrowser,
  THEME_PROVIDER,
} from '@openmina/shared';
import { CommonModule, registerLocaleData } from '@angular/common';
import localeFr from '@angular/common/locales/fr';
import localeEn from '@angular/common/locales/en';
import { MenuComponent } from '@app/layout/menu/menu.component';
import { ToolbarComponent } from '@app/layout/toolbar/toolbar.component';
import { ErrorPreviewComponent } from '@error-preview/error-preview.component';
import { ErrorListComponent } from '@error-preview/error-list/error-list.component';
import { ServerStatusComponent } from '@app/layout/server-status/server-status.component';
import { SubmenuTabsComponent } from '@app/layout/submenu-tabs/submenu-tabs.component';
import { NodePickerComponent } from '@app/layout/node-picker/node-picker.component';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import { CONFIG } from '@shared/constants/config';
import { StoreModule } from '@ngrx/store';
import { metaReducers, reducers } from '@app/app.setup';
import { EffectsModule } from '@ngrx/effects';
import { AppEffects } from '@app/app.effects';
import { StoreDevtoolsModule } from '@ngrx/store-devtools';
import { HttpClientModule, provideHttpClient, withFetch } from '@angular/common/http';
import { NewNodeComponent } from './layout/new-node/new-node.component';
import { ReactiveFormsModule } from '@angular/forms';
import { WebNodeLandingPageComponent } from '@app/layout/web-node-landing-page/web-node-landing-page.component';
import * as Sentry from '@sentry/angular';
import { Router } from '@angular/router';
import { getApp, initializeApp, provideFirebaseApp } from '@angular/fire/app';
import { getAnalytics, provideAnalytics, ScreenTrackingService } from '@angular/fire/analytics';
import { getPerformance, providePerformance } from '@angular/fire/performance';
import { BlockProductionPillComponent } from '@app/layout/block-production-pill/block-production-pill.component';
import { MenuTabsComponent } from '@app/layout/menu-tabs/menu-tabs.component';
import { getFirestore, provideFirestore } from '@angular/fire/firestore';
import { LeaderboardModule } from '@leaderboard/leaderboard.module';
import { UptimePillComponent } from '@app/layout/uptime-pill/uptime-pill.component';
import { provideAppCheck } from '@angular/fire/app-check';
import { initializeAppCheck, ReCaptchaV3Provider } from 'firebase/app-check';

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
    window.addEventListener('error', (event: ErrorEvent) => {
      event.preventDefault();
      this.handleError(event.error);
    }, { capture: true });

    // Override console.error with proper error extraction
    const originalConsoleError = console.error;
    console.error = (...args) => {
      // Find the actual error object in the arguments
      const error = args.find(arg => arg instanceof Error) ||
        args.join(' ');

      this.handleError(error);
      originalConsoleError.apply(console, args);
    };
  }

  private interceptWebAssembly(): void {
    const self = this;

    const originalInstantiateStreaming = WebAssembly.instantiateStreaming;
    if (originalInstantiateStreaming) {
      WebAssembly.instantiateStreaming = async function (response: any, importObject?: any): Promise<any> {
        try {
          return await originalInstantiateStreaming.call(WebAssembly, response, importObject);
        } catch (error) {
          self.handleError(error);
          throw error;
        }
      };
    }

    const originalInstantiate = WebAssembly.instantiate;
    WebAssembly.instantiate = async function (moduleObject: any, importObject?: any): Promise<any> {
      try {
        return await originalInstantiate.call(WebAssembly, moduleObject, importObject);
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

const firebaseProviders = [
  provideFirebaseApp(() => initializeApp(CONFIG.globalConfig.firebase)),
  provideClientHydration(),
  provideHttpClient(withFetch()),
  provideAnalytics(() => getAnalytics()),
  ScreenTrackingService,
  // provideAppCheck(() => {
  //   // TODO get a reCAPTCHA Enterprise here https://console.cloud.google.com/security/recaptcha?project=_
  //   const app = getApp();
  //   const provider = new ReCaptchaV3Provider('6LfAB-QqAAAAAEu9BO6upFj6Sewd08lf0UtFC16c');
  //   return initializeAppCheck(app, { provider, isTokenAutoRefreshEnabled: true });
  // }),
  providePerformance(() => getPerformance()),
  provideFirestore(() => getFirestore()),
];

@NgModule({
  declarations: [
    AppComponent,
    MenuComponent,
    ToolbarComponent,
    ErrorPreviewComponent,
    ErrorListComponent,
    ServerStatusComponent,
    SubmenuTabsComponent,
    NodePickerComponent,
    NewNodeComponent,
  ],
  imports: [
    CommonModule,
    BrowserModule,
    BrowserAnimationsModule,
    AppRouting,
    StoreModule.forRoot(reducers, {
      metaReducers,
      runtimeChecks: {
        strictStateImmutability: true,
        strictActionImmutability: true,
        strictActionWithinNgZone: true,
        strictStateSerializability: true,
      },
    }),
    EffectsModule.forRoot([AppEffects]),
    NgrxRouterStoreModule,
    !CONFIG.production ? StoreDevtoolsModule.instrument({ maxAge: 150, connectInZone: true }) : [],
    HttpClientModule,
    MatSidenavModule,
    OpenminaEagerSharedModule,
    HorizontalMenuComponent,
    ReactiveFormsModule,
    CopyComponent,
    WebNodeLandingPageComponent,
    BlockProductionPillComponent,
    MenuTabsComponent,
    LeaderboardModule,
    UptimePillComponent,
  ],
  providers: [
    THEME_PROVIDER,
    { provide: LOCALE_ID, useValue: 'en' },
    { provide: ErrorHandler, useValue: Sentry.createErrorHandler() },
    { provide: ErrorHandler, useClass: AppGlobalErrorhandler, deps: [GlobalErrorHandlerService], multi: false },
    { provide: Sentry.TraceService, deps: [Router] },
    {
      provide: APP_INITIALIZER,
      useFactory: () => () => {
      },
      deps: [Sentry.TraceService],
      multi: true,
    },
    ...[CONFIG.globalConfig.firebase ? firebaseProviders : []],
  ],
  bootstrap: [AppComponent],
  exports: [
    MenuComponent,
  ],
})
export class AppModule {
}
