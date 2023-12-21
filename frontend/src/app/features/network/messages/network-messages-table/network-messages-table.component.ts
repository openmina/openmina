import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { NetworkMessage } from '@shared/types/network/messages/network-message.type';
import {
  NETWORK_GET_SPECIFIC_MESSAGE,
  NETWORK_PAUSE,
  NETWORK_SET_ACTIVE_ROW,
  NETWORK_SET_TIMESTAMP_INTERVAL,
  NETWORK_TOGGLE_FILTER,
  NetworkMessagesGetMessages,
  NetworkMessagesGetSpecificMessage,
  NetworkMessagesPause,
  NetworkMessagesSetActiveRow,
  NetworkMessagesSetTimestampInterval,
  NetworkMessagesToggleFilter,
} from '@network/messages/network-messages.actions';
import {
  selectNetworkActiveFilters,
  selectNetworkActiveRow,
  selectNetworkMessages,
  selectNetworkStream
} from '@network/messages/network-messages.state';
import { untilDestroyed } from '@ngneat/until-destroy';
import { NetworkMessagesFilter } from '@shared/types/network/messages/network-messages-filter.type';
import { NetworkMessagesFilterTypes } from '@shared/types/network/messages/network-messages-filter-types.enum';
import { filter, fromEvent, take, throttleTime } from 'rxjs';
import { NetworkMessagesFilterCategory } from '@shared/types/network/messages/network-messages-filter-group.type';
import {
  networkAvailableFilters
} from '@network/messages/network-messages-filters/network-messages-filters.component';
import { getMergedRoute, lastItem, MergedRoute, TableColumnList, TimestampInterval } from '@openmina/shared';
import { Params, Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { NetworkMessagesDirection } from '@shared/types/network/messages/network-messages-direction.enum';
// import { APP_UPDATE_DEBUGGER_STATUS, AppUpdateDebuggerStatus } from '@app/app.actions';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-network-messages-table',
  templateUrl: './network-messages-table.component.html',
  styleUrls: ['./network-messages-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkMessagesTableComponent extends MinaTableRustWrapper<NetworkMessage> implements OnInit {

  protected readonly tableHeads: TableColumnList<NetworkMessage> = [
    { name: 'ID' },
    { name: 'datetime' },
    { name: 'remote address' },
    { name: 'direction' },
    { name: 'size' },
    { name: 'stream kind' },
    { name: 'message kind' },
  ];

  messages: NetworkMessage[] = [];
  activeFilters: NetworkMessagesFilter[] = [];
  attemptToGetMessagesFromRoute: boolean = true;

  private idFromRoute: number;
  private activeRow: NetworkMessage;
  private stream: boolean;
  private queryParams: Params;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToNetworkMessages();
    this.listenToActiveRowChange();
    this.listenToNetworkFilters();
    this.listenToNetworkStream();
    this.listenToVirtualScrolling();
    this.listenToRouteChange();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [80, 170, 190, 100, 80, 140, 400];
    this.table.propertyForActiveCheck = 'id';
  }

  private listenToRouteChange(): void {
    this.store.select(getMergedRoute)
      .pipe(untilDestroyed(this), take(1))
      .subscribe((route: MergedRoute) => {
        this.queryParams = route.queryParams;
        const idFromRoute = Number(route.params['messageId']);
        const isValidIdInRoute = !isNaN(idFromRoute);
        if (this.attemptToGetMessagesFromRoute && (isValidIdInRoute || Object.keys(route.queryParams).filter(key => key !== 'node').length !== 0)) {
          const filters = this.getFiltersFromTheRoute(route);

          const timestamp: TimestampInterval = {
            from: Number(route.queryParams['from']),
            to: Number(route.queryParams['to']),
          };
          const direction = route.queryParams['from'] ? NetworkMessagesDirection.FORWARD : undefined;

          if (isValidIdInRoute) {
            this.idFromRoute = idFromRoute;
            this.store.dispatch<NetworkMessagesGetSpecificMessage>({
              type: NETWORK_GET_SPECIFIC_MESSAGE,
              payload: { id: idFromRoute, filters, type: 'add', timestamp, direction },
            });
          } else if (filters.length) {
            this.store.dispatch<NetworkMessagesToggleFilter>({
              type: NETWORK_TOGGLE_FILTER,
              payload: { filters, type: 'add', timestamp, direction },
            });
          } else {
            this.store.dispatch<NetworkMessagesSetTimestampInterval>({
              type: NETWORK_SET_TIMESTAMP_INTERVAL,
              payload: { timestamp, direction },
            });
          }
        } else {
          this.dispatch(NetworkMessagesGetMessages);
        }
        this.attemptToGetMessagesFromRoute = false;
      });
  }

  private getFiltersFromTheRoute(route: MergedRoute): NetworkMessagesFilter[] {
    const filters: NetworkMessagesFilter[] = [];
    const availableFilters: NetworkMessagesFilter[] = networkAvailableFilters
      .reduce((acc: NetworkMessagesFilter[], current: NetworkMessagesFilterCategory[]) => [
        ...acc,
        ...current.reduce((acc2: NetworkMessagesFilter[], curr: NetworkMessagesFilterCategory) => [...acc2, ...curr.filters], []),
      ], []);
    const streamKindFilters = route.queryParams['stream_kind']?.split(',').map((value: string) => availableFilters.find(f => f.value === value)) ?? [];

    const messageKindFilters = route.queryParams['message_kind']?.split(',').map((value: string) => availableFilters.find(f => f.value === value)) ?? [];

    const address = route.queryParams['addr'];
    if (address) {
      filters.push({
        type: NetworkMessagesFilterTypes.ADDRESS,
        value: address,
        display: address,
      } as NetworkMessagesFilter);
    }
    filters.push(...streamKindFilters);
    filters.push(...messageKindFilters);
    return filters;
  }

  private listenToNetworkMessages(): void {
    this.store.select(selectNetworkMessages)
      .pipe(untilDestroyed(this))
      .subscribe((messages: NetworkMessage[]) => {
        this.messages = messages;
        this.table.rows = messages;
        this.table.detect();
        this.detect();
        this.scrollToElement();
        this.sendTotalDecrypted();
      });
  }

  private scrollToElement(): void {
    let rowFinder = (r: NetworkMessage) => r.id === this.idFromRoute;
    if (!this.idFromRoute) {
      rowFinder = (r: NetworkMessage) => r.id === lastItem(this.table.rows).id;
    }
    this.table.scrollToElement(rowFinder);
    delete this.idFromRoute;
  }

  private listenToNetworkStream(): void {
    this.store.select(selectNetworkStream)
      .pipe(untilDestroyed(this))
      .subscribe((stream: boolean) => this.stream = stream);
  }

  private listenToActiveRowChange(): void {
    this.store.select(selectNetworkActiveRow)
      .pipe(untilDestroyed(this))
      .subscribe((row: NetworkMessage) => {
        this.activeRow = row;
        this.table.activeRow = row;
        this.table.detect();
        this.detect();
      });
  }

  private listenToNetworkFilters(): void {
    this.store.select(selectNetworkActiveFilters)
      .pipe(untilDestroyed(this))
      .subscribe((activeFilters: NetworkMessagesFilter[]) => {
        this.activeFilters = activeFilters;
      });
  }

  private listenToVirtualScrolling(): void {
    fromEvent(this.table.virtualScroll.elementRef.nativeElement.firstChild, 'wheel', { passive: true })
      .pipe(
        untilDestroyed(this),
        throttleTime(600),
        filter((event: Event) => this.stream && (event as WheelEvent).deltaY < 0),
      )
      .subscribe(() => this.pause());
    fromEvent(this.table.virtualScroll.elementRef.nativeElement, 'touchmove', { passive: true })
      .pipe(
        untilDestroyed(this),
        throttleTime(600),
        filter(() => this.stream),
      )
      .subscribe(() => this.pause());
  }

  protected override onRowClick(row: NetworkMessage): void {
    if (row.id !== this.activeRow?.id) {
      this.router.navigate([Routes.NETWORK, Routes.MESSAGES, row.id], { queryParamsHandling: 'merge' });
      this.store.dispatch<NetworkMessagesSetActiveRow>({ type: NETWORK_SET_ACTIVE_ROW, payload: row });
    }
  }

  filterByAddress(message: NetworkMessage): void {
    const type = this.activeFilters.some(f => f.value === message.address) ? 'remove' : 'add';
    const filter = NetworkMessagesTableComponent.getFilter(message);
    this.sendFilterAction([filter], type);
  }

  private static getFilter(message: NetworkMessage): NetworkMessagesFilter {
    return {
      type: NetworkMessagesFilterTypes.ADDRESS,
      display: message.address,
      value: message.address,
    };
  }

  private sendFilterAction(filters: NetworkMessagesFilter[], type: 'remove' | 'add'): void {
    this.store.dispatch<NetworkMessagesToggleFilter>({ type: NETWORK_TOGGLE_FILTER, payload: { filters, type } });
  }

  private pause(): void {
    this.store.dispatch<NetworkMessagesPause>({ type: NETWORK_PAUSE });
  }

  clearFilters(): void {
    this.sendFilterAction(this.activeFilters, 'remove');
  }

  private sendTotalDecrypted(): void {
    if (!this.messages.length) {
      return;
    }

    const current: NetworkMessage = this.messages.slice().reverse().find((m: NetworkMessage) => m.failedToDecryptPercentage !== undefined);
    if (current) {
      // todo: check
      // this.store.dispatch<AppUpdateDebuggerStatus>({
      //   type: APP_UPDATE_DEBUGGER_STATUS,
      //   payload: { failed: current.failedToDecryptPercentage },
      // });
    }
  }
}
