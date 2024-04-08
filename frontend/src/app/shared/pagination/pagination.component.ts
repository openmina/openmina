import {
  ChangeDetectionStrategy,
  Component,
  EventEmitter,
  Input,
  OnChanges,
  Output,
  SimpleChanges,
} from '@angular/core';
import { SharedModule } from '@shared/shared.module';

@Component({
  selector: 'mina-pagination',
  standalone: true,
  imports: [SharedModule],
  templateUrl: './pagination.component.html',
  styleUrls: ['./pagination.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'fx-row-vert-cent' },
})
export class PaginationComponent implements OnChanges {

  @Input({ required: true }) activePage: number;
  /**
   * @description if provided, will be used as display instead of activePage
   */
  @Input() activePageText: string;
  @Input() prevPageDisabled: boolean;
  @Input() prevPageTooltip: string;
  @Input() nextPageDisabled: boolean;
  @Input() nextPageTooltip: string;
  @Input() hasLastPage: boolean;
  @Input() lastPageDisabled: boolean;
  @Input() lastPageTooltip: string;
  @Input() hasFirstPage: boolean;
  @Input() firstPageDisabled: boolean;
  @Input() firstPageTooltip: string;
  /**
   * @description min-width of the active page element.
   *
   * If activePageText is provided, this will be used,
   * otherwise, it will be calculated based on the number of digits in activePage.
   */
  @Input() minWidth: number = 50;

  @Output() prevPageChange: EventEmitter<number> = new EventEmitter<number>();
  @Output() nextPageChange: EventEmitter<number> = new EventEmitter<number>();
  @Output() lastPageChange: EventEmitter<void> = new EventEmitter<void>();
  @Output() firstPageChange: EventEmitter<void> = new EventEmitter<void>();

  computedMinWidth: number;

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['activePage'] || changes['activePageText'] || changes['minWidth']) {
      this.computedMinWidth = this.activePageText ? this.minWidth : this.activePage?.toString().length * 10;
    }
  }
}
