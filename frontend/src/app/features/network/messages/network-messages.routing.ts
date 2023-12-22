import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { NetworkMessagesComponent } from '@network/messages/network-messages.component';
import { NETWORK_TITLE } from '@app/app.routing';

const routes: Routes = [
  {
    path: '',
    component: NetworkMessagesComponent,
    children: [
      {
        path: ':messageId',
        component: NetworkMessagesComponent,
        title: NETWORK_TITLE,
      },
    ],
  },
  {
    path: '**',
    redirectTo: '',
    pathMatch: 'full',
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
})
export class NetworkMessagesRouting {}
