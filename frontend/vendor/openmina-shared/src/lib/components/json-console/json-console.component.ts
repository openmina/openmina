import { Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { OpenminaEagerSharedModule } from '../../openmina-eager-shared.module';
import { ExpandTracking, MinaJsonViewerComponent } from '../mina-json-viewer/mina-json-viewer.component';
import { downloadJson } from '../../helpers/user-input.helper';
import { REQUIRED } from '../../constants/angular';
import { HorizontalMenuComponent } from '../horizontal-menu/horizontal-menu.component';

@Component({
    selector: 'mina-json-console',
    imports: [MinaJsonViewerComponent, OpenminaEagerSharedModule, CommonModule, HorizontalMenuComponent],
    templateUrl: './json-console.component.html',
    styleUrls: ['./json-console.component.scss']
})
export class JsonConsoleComponent implements OnChanges {

  @Input(REQUIRED) json: object | string | number | Array<any> | null | undefined;
  @Input(REQUIRED) fileName: string;
  @Input() expanded: boolean = false;
  @Input() saveBinBtn: boolean = false;
  @Input() keepExpandingHistory: boolean = true;

  @Output() onDownloadBin: EventEmitter<void> = new EventEmitter<void>();

  jsonString: string;
  expandTracking: ExpandTracking = {};

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['json']) {
      setTimeout(() => this.jsonString = JSON.stringify(this.json), 400);
    }
  }

  downloadJson(): void {
    downloadJson(this.jsonString, this.fileName + '.json');
  }

  expandEntireJSON(): void {
    this.expandTracking = this.minaJsonViewer.toggleAll(true);
  }

  collapseEntireJSON(): void {
    this.expandTracking = this.minaJsonViewer.toggleAll(false);
  }

  downloadBin(): void {
    this.onDownloadBin.emit();
  }
}
