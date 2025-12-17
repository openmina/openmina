import { OverlayModule } from '@angular/cdk/overlay';
import { NgModule } from '@angular/core';
import { ClickOutsideDirective } from './directives/click-outside.directive';
import { MinaTooltipDirective } from './directives/mina-tooltip.directive';
import { CopyToClipboardDirective } from './directives/copy-to-clipboard.directive';
import { ClipboardModule } from '@angular/cdk/clipboard';
import { PluralPipe } from './pipes/plural.pipe';

const EAGER_MODULES = [
  OverlayModule,
  ClipboardModule,
];

const EAGER_DIRECTIVES = [
  ClickOutsideDirective,
  MinaTooltipDirective,
  CopyToClipboardDirective,
];

const EAGER_PIPES = [
  PluralPipe,
];


/**
 * @description
 * The role of this module is to eagerly load all shared parts that are also required by AppModule.
 * */
@NgModule({
  imports: [
    ...EAGER_MODULES,
    ...EAGER_PIPES,
  ],
  exports: [
    ...EAGER_MODULES,
    ...EAGER_DIRECTIVES,
    ...EAGER_PIPES,
  ],
  declarations: [
    ...EAGER_DIRECTIVES,
  ],
})
export class OpenminaEagerSharedModule {}
