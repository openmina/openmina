import { ChangeDetectionStrategy, Component, OnDestroy, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { MemoryResourcesClose, MemoryResourcesGet, MemoryResourcesSetActiveResource } from '@resources/memory/memory-resources.actions';
import { selectActiveNode } from '@app/app.state';
import { filter } from 'rxjs';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { selectMemoryResourcesState } from '@resources/resources.state';
import { HorizontalMenuComponent } from '@openmina/shared';

@Component({
  selector: 'app-memory-resources',
  templateUrl: './memory-resources.component.html',
  styleUrls: ['./memory-resources.component.scss'],
  host: { class: 'flex-column h-100 pb-10' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MemoryResourcesComponent extends StoreDispatcher implements OnInit, OnDestroy {

  activeResource: MemoryResource;
  breadcrumbs: MemoryResource[];

  @ViewChild(HorizontalMenuComponent) private horizontalMenu: HorizontalMenuComponent;

  ngOnInit(): void {
    this.listenToActiveNodeChange();
    this.listenToActiveResource();
  }

  private listenToActiveNodeChange(): void {
    this.select(selectActiveNode, () => {
      this.dispatch(MemoryResourcesGet);
    }, filter(Boolean));
  }

  private listenToActiveResource(): void {
    this.select(selectMemoryResourcesState, (state: MemoryResourcesState) => {
      this.activeResource = state.activeResource;
      this.breadcrumbs = state.breadcrumbs;
      this.detect();
      this.horizontalMenu.checkView();
      this.horizontalMenu.scrollRight();
    }, filter(s => !!s.resource));
  }

  setActiveResource(breadcrumb: MemoryResource): void {
    this.dispatch(MemoryResourcesSetActiveResource, breadcrumb);
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(MemoryResourcesClose);
  }
}
