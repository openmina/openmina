import { ErrorHandler, Injectable, LOCALE_ID, NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppComponent } from './app.component';
import { AppRouting } from './app.routing';
import { MatSidenavModule } from '@angular/material/sidenav';
import {
  CopyComponent,
  GlobalErrorHandlerService,
  HorizontalMenuComponent,
  NgrxRouterStoreModule,
  OpenminaEagerSharedModule,
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
import { HttpClientModule } from '@angular/common/http';
import { NewNodeComponent } from './layout/new-node/new-node.component';
import { ReactiveFormsModule } from '@angular/forms';

registerLocaleData(localeFr, 'fr');
registerLocaleData(localeEn, 'en');

@Injectable()
export class AppGlobalErrorhandler implements ErrorHandler {
  constructor(private errorHandlerService: GlobalErrorHandlerService) {}

  handleError(error: any) {
    this.errorHandlerService.handleError(error);
    console.error(error);
  }
}

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
    !CONFIG.production ? StoreDevtoolsModule.instrument({ maxAge: 150 }) : [],
    HttpClientModule,
    MatSidenavModule,
    OpenminaEagerSharedModule,
    HorizontalMenuComponent,
    ReactiveFormsModule,
    CopyComponent,
  ],
  providers: [
    THEME_PROVIDER,
    { provide: ErrorHandler, useClass: AppGlobalErrorhandler, deps: [GlobalErrorHandlerService] },
    { provide: LOCALE_ID, useValue: 'en' },
  ],
  bootstrap: [AppComponent],
})
export class AppModule {}
