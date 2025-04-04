import { CanActivateFn, Router } from '@angular/router';
import { inject } from '@angular/core';
import { Store } from '@ngrx/store';
import { map, take } from 'rxjs/operators';
import { CONFIG } from '@shared/constants/config';
import { getMergedRoute } from '@openmina/shared';
import { Routes } from '@shared/enums/routes.enum';

let isFirstLoad = true;

export const landingPageGuard: CanActivateFn = (route, state) => {

  if (!isFirstLoad || !CONFIG.showWebNodeLandingPage) {
    return true;
  }
  const router = inject(Router);
  const store = inject(Store);
  isFirstLoad = false;

  return store.select(getMergedRoute).pipe(
    take(1),
    map(route => {
      if (!route) return true;

      const startsWith = (path: string) => route.url.startsWith(path);

      if (
        startsWith('/dashboard') ||
        startsWith('/block-production') ||
        startsWith('/state') ||
        startsWith('/mempool') ||
        startsWith('/loading-web-node')
      ) {
        return router.createUrlTree([Routes.LOADING_WEB_NODE], {
          queryParamsHandling: 'preserve',
        });
      }

      if (!startsWith('/') && !startsWith('/?') && !startsWith('/leaderboard')) {
        return router.createUrlTree(['']);
      }

      return true;
    }),
  );
};
