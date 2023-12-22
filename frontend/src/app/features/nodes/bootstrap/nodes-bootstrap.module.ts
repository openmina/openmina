import { NgModule } from '@angular/core';

import { NodesBootstrapRouting } from './nodes-bootstrap.routing';
import { NodesBootstrapComponent } from './nodes-bootstrap.component';
import { NodesBootstrapTableComponent } from '@nodes/bootstrap/nodes-bootstrap-table/nodes-bootstrap-table.component';
import { NodesBootstrapSidePanelComponent } from '@nodes/bootstrap/nodes-bootstrap-side-panel/nodes-bootstrap-side-panel.component';
import { SharedModule } from '@shared/shared.module';
import { EffectsModule } from '@ngrx/effects';
import { NodesBootstrapEffects } from '@nodes/bootstrap/nodes-bootstrap.effects';
import { NodesBootstrapOverviewComponent } from '@nodes/bootstrap/nodes-bootstrap-overview/nodes-bootstrap-overview.component';
import { NodesBootstrapBlocksComponent } from '@nodes/bootstrap/nodes-bootstrap-blocks/nodes-bootstrap-blocks.component';
import { CopyComponent, HorizontalResizableContainerComponent, MinaJsonViewerComponent } from '@openmina/shared';


@NgModule({
  declarations: [
    NodesBootstrapComponent,
    NodesBootstrapTableComponent,
    NodesBootstrapSidePanelComponent,
    NodesBootstrapOverviewComponent,
    NodesBootstrapBlocksComponent,
  ],
  imports: [
    SharedModule,
    NodesBootstrapRouting,
    HorizontalResizableContainerComponent,
    EffectsModule.forFeature(NodesBootstrapEffects),
    CopyComponent,
    MinaJsonViewerComponent,
  ],
})
export class NodesBootstrapModule {}
