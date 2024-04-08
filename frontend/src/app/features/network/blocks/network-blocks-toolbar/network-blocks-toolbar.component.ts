import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import {
  selectNetworkBlocksActiveBlock,
  selectNetworkBlocksActiveFilters,
  selectNetworkBlocksAllFilters,
  selectNetworkBlocksEarliestBlock,
} from '@network/blocks/network-blocks.state';
import {
  NetworkBlocksSetActiveBlock,
  NetworkBlocksToggleFilter,
  NetworkBlocksToggleSidePanel,
} from '@network/blocks/network-blocks.actions';
import { filter } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { Router } from '@angular/router';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-network-blocks-toolbar',
  templateUrl: './network-blocks-toolbar.component.html',
  styleUrls: ['./network-blocks-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'border-bottom flex-column' },
})
export class NetworkBlocksToolbarComponent extends StoreDispatcher implements OnInit {

  activeFilters: string[] = [];
  allFilters: string[] = [];
  activeBlock: number;
  earliestBlock: number;

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToFiltersChanges();
    this.listenToActiveBlockChanges();
  }

  toggleFilter(filter: string): void {
    this.dispatch(NetworkBlocksToggleFilter, filter);
  }

  toggleSidePanel(): void {
    this.dispatch(NetworkBlocksToggleSidePanel);
  }

  getBlock(height: number): void {
    this.dispatch(NetworkBlocksSetActiveBlock, { height, fetchNew: true });
    this.router.navigate([Routes.NETWORK, Routes.BLOCKS, height], { queryParamsHandling: 'merge' });
  }

  private listenToFiltersChanges(): void {
    this.select(selectNetworkBlocksAllFilters, (filters: string[]) => {
      this.allFilters = filters;
      this.detect();
    });
    this.select(selectNetworkBlocksActiveFilters, (filters: string[]) => {
      this.activeFilters = filters;
      this.detect();
    });
  }

  private listenToActiveBlockChanges(): void {
    this.select(selectNetworkBlocksActiveBlock, (block: number) => {
      this.activeBlock = block;
      this.detect();
    });

    this.select(selectNetworkBlocksEarliestBlock, (earliestBlock: number) => {
      this.earliestBlock = earliestBlock;
      this.detect();
    }, filter(Boolean), filter(earliestBlock => this.earliestBlock !== earliestBlock));
  }
}
