import { NgModule } from '@angular/core';
import { OpenminaEagerSharedModule } from './openmina-eager-shared.module';
import { SizePipe } from './pipes/size.pipe';
import { TruncateMidPipe } from './pipes/truncate-mid.pipe';
import { SecDurationPipe } from './pipes/sec-duration.pipe';
import { ThousandPipe } from './pipes/thousand.pipe';
import { ReadableDatePipe } from './pipes/readable-date.pipe';
import { PluralPipe } from './pipes/plural.pipe';
import { SafeHtmlPipe } from './pipes/safe-html.pipe';

const PIPES = [
  SizePipe,
  TruncateMidPipe,
  SecDurationPipe,
  ThousandPipe,
  ReadableDatePipe,
  SafeHtmlPipe,
];

const MODULES = [
  OpenminaEagerSharedModule,
];

@NgModule({
  declarations: [
    ...PIPES,
  ],
  imports: [
    ...MODULES,
  ],
  exports: [
    ...MODULES,
    ...PIPES,
  ]
})
export class OpenminaSharedModule {}
