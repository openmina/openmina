import { ActivatedRouteSnapshot, CanActivateFn, Router } from '@angular/router';
import { inject } from '@angular/core';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectActiveNode } from '@app/app.state';
import { filter, map } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { getFeaturesConfig } from '@shared/constants/config';

export const networkGuard: CanActivateFn = (route: ActivatedRouteSnapshot) => {
  const store: Store<MinaState> = inject<Store<MinaState>>(Store<MinaState>);
  const router: Router = inject<Router>(Router);
  return store.select(selectActiveNode).pipe(
    filter(Boolean),
    map((node: MinaNode) => {
      const currentRoute = route.routeConfig.path;
      const features = getFeaturesConfig(node);
      if (features['network'].includes(currentRoute)) {
        return true;
      }
      router.navigate([Routes.NETWORK, getFeaturesConfig(node)['network'][0]]);
      return false;
    }),
  );
};
