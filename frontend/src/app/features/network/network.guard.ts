import { ActivatedRouteSnapshot, CanActivateFn, Router } from '@angular/router';
import { inject } from '@angular/core';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { AppSelectors } from '@app/app.state';
import { filter, map } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { FeaturesConfig, MinaNode } from '@shared/types/core/environment/mina-env.type';
import { getFeaturesConfig, getFirstFeature } from '@shared/constants/config';

export const networkGuard: CanActivateFn = (route: ActivatedRouteSnapshot) => {
  const store: Store<MinaState> = inject<Store<MinaState>>(Store<MinaState>);
  const router: Router = inject<Router>(Router);
  return store.select(AppSelectors.activeNode).pipe(
    filter(Boolean),
    map((node: MinaNode): boolean => {
      const currentRoute: string = route.routeConfig.path;
      const features: FeaturesConfig = getFeaturesConfig(node);
      if (features['network']?.includes(currentRoute)) {
        return true;
      } else if (!features['network']) {
        router.navigate([getFirstFeature(node)]);
        return false;
      }
      router.navigate([Routes.NETWORK, getFeaturesConfig(node)['network'][0]]);
      return false;
    }),
  );
};
