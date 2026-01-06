import { ChangeDetectionStrategy, Component, Input, OnChanges, OnInit } from '@angular/core';
import { NgxJsonViewerComponent, NgxJsonViewerModule } from 'ngx-json-viewer';
import { Segment } from 'ngx-json-viewer/src/ngx-json-viewer/ngx-json-viewer.component';
import { CommonModule } from '@angular/common';

export interface ExpandTracking {
  [p: string]: ExpandTracking;
}

@Component({
    selector: 'mina-json-viewer',
    templateUrl: './mina-json-viewer.component.html',
    styleUrls: ['./mina-json-viewer.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    imports: [CommonModule, NgxJsonViewerModule]
})
export class MinaJsonViewerComponent extends NgxJsonViewerComponent implements OnChanges, OnInit {

  @Input() paddingLimitForNestedElements: number = 1000;
  @Input() internalDepth: number = 0;
  @Input() expandTracking: ExpandTracking;
  @Input() skipDecycle: boolean = false;

  ngOnInit(): void {
    this.internalDepth++;
  }

  override ngOnChanges(): void {
    this.segments = [];

    if (this.skipDecycle) {
      this.json = this['decycle'](this.json);
    }

    this['_currentDepth']++;

    if (typeof this.json === 'object') {
      Object.keys(this.json).forEach(key => {
        this.segments.push(this.customParseKeyValue(key, this.json[key]));
      });
    } else {
      this.segments.push(this.customParseKeyValue(`(${typeof this.json})`, this.json));
    }

    if (this.expandTracking) {
      Object.keys(this.expandTracking).forEach((key: string) => {
        const segmentToToggle: Segment = this.segments.find(segment => key === segment.key);
        if (segmentToToggle && !segmentToToggle.expanded) {
          this.toggle(segmentToToggle);
        }
      });
    }
  }

  toggleAll(expand: boolean): ExpandTracking {
    const newExpandTracking = {};
    this.segments.forEach((segment: Segment) => {
      segment.expanded = expand;
      if (expand) {
        this.appendExpandingRecursively(segment.value, segment.key, newExpandTracking);
      }
    });
    return newExpandTracking;
  }

  onExpandToggle(segment: Segment): void {
    this.toggle(segment);
    this.appendExpandingToExpandTrackingElement(segment);
  }

  private appendExpandingToExpandTrackingElement(segment: Segment): void {
    if (!this.expandTracking || !this.isExpandable(segment)) {
      return;
    }

    if (this.expandTracking[segment.key]) {
      delete this.expandTracking[segment.key];
    } else {
      this.expandTracking[segment.key] = {};
    }
  }

  private customParseKeyValue(key: any, value: any): Segment {
    /*const segment: Segment =*/
    return this['parseKeyValue'](key, value);
    // if (typeof segment.value === 'string' && Number(segment.value) <= 10000000000000000000 && Number(segment.value) >= 10000000000000000) {
    //   segment.type = 'number';
    //   segment.description = '' + segment.value;
    // }
    // return segment;
  }

  private appendExpandingRecursively(value: any, keyOfValue: string, tracking: ExpandTracking): void {
    const isExpandable = (v: any) => typeof v === 'object' || Array.isArray(v);
    if (value !== null && isExpandable(value)) {
      tracking[keyOfValue] = {};
      Object.keys(value).forEach(key => {
        this.appendExpandingRecursively(value[key], key, tracking[keyOfValue]);
      });
    }
  }
}
