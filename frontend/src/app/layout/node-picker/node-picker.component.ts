import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, ViewChild } from '@angular/core';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { debounceTime, distinctUntilChanged, filter, fromEvent, map } from 'rxjs';
import { AppAddNode, AppChangeActiveNode } from '@app/app.actions';
import { untilDestroyed } from '@ngneat/until-destroy';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-node-picker',
  templateUrl: './node-picker.component.html',
  styleUrls: ['./node-picker.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100' },
})
export class NodePickerComponent extends StoreDispatcher implements AfterViewInit {

  activeNode: MinaNode;
  nodes: MinaNode[] = [];

  filteredNodes: MinaNode[] = [];
  closeEmitter: EventEmitter<void> = new EventEmitter<void>();
  parentInitialWidth: number = 0;

  @ViewChild('searchNode') searchInput: ElementRef<HTMLInputElement>;

  constructor(private elementRef: ElementRef<HTMLElement>) { super(); }

  ngAfterViewInit(): void {
    this.listenToNodeSearch();
    this.parentInitialWidth = this.elementRef.nativeElement.offsetWidth;
    this.searchInput.nativeElement.focus();
    this.detect();
  }

  private listenToNodeSearch(): void {
    fromEvent(this.searchInput.nativeElement, 'keyup')
      .pipe(
        untilDestroyed(this),
        debounceTime(50),
        distinctUntilChanged(),
        filter(() => {
          if (this.searchInput.nativeElement.value.length <= 1) {
            this.filteredNodes = this.nodes;
            this.detect();
            return false;
          }
          return this.searchInput.nativeElement.value.length > 1;
        }),
        map(() => this.searchInput.nativeElement.value.toLowerCase()),
      )
      .subscribe((value: string) => {
        this.filteredNodes = this.nodes.filter(n => n.name.toLowerCase().includes(value) || n.url.toLowerCase().includes(value));
        this.detect();
      });
  }

  selectNode(node: MinaNode): void {
    this.closeEmitter.emit();
    if (node !== this.activeNode) {
      this.dispatch(AppChangeActiveNode, node);
    }
  }

  addNode(event: MouseEvent): void {
    event.stopImmediatePropagation();
    const value = this.searchInput.nativeElement.value.trim();
    const payload = value[value.length - 1] === '/' ? value.slice(0, -1) : value;
    this.dispatch(AppAddNode, payload);
    this.searchInput.nativeElement.value = '';
  }
}
