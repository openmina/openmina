/// <reference types="cypress" />
// ***********************************************
// This example commands.ts shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************


import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { map } from 'rxjs';
import { FeaturesConfig, FeatureType, MinaNode } from '@shared/types/core/environment/mina-env.type';

export const stateSliceAsPromise = <T = MinaState | MinaState[keyof MinaState]>(
  store: Store<MinaState>, resolveCondition: (state: T) => boolean, slice: keyof MinaState, subSlice?: string, timeout: number = 3000,
): T => {
  return new Cypress.Promise((resolve: (result?: T | void) => void): void => {
    const observer = (state: T) => {
      if (resolveCondition(state)) {
        return resolve(state);
      }
      setTimeout(() => resolve(), timeout);
    };
    store.select(slice).pipe(
      map((subState: MinaState[keyof MinaState]) => {
        // cy.log('');
        return subSlice ? (subState as any)[subSlice] : subState;
      }),
    ).subscribe(observer);
  }) as T;
};

// export const storeNetworkSubscription = (store: Store<MinaState>, observer: any): Subscription => store.select('network').subscribe(observer);

Cypress.Commands.overwrite('log', (subject, message) => cy.task('log', message));

export enum Sort {
  ASC = 'asc',
  DSC = 'desc',
}

export function checkSorting<T>(array: T[], field: keyof T, direction: Sort): void {
  let sorted = true;
  const isStringProp = typeof array[0][field] === 'string';
  for (let i = 0; i < array.length - 1; i++) {
    if (isStringProp) {
      const curr: string = array[i][field] as string || '';
      const next: string = array[i + 1][field] as string || '';
      if ((direction === Sort.DSC && curr.localeCompare(next) < 0) || (direction === Sort.ASC && curr.localeCompare(next) > 0)) {
        sorted = false;
        break;
      }
    } else {
      const curr = array[i][field] === undefined ? array[i][field] : Number.MAX_VALUE;
      const next = array[i + 1][field] === undefined ? array[i + 1][field] : Number.MAX_VALUE;
      if ((direction === Sort.DSC && curr > next) || (direction === Sort.ASC && curr < next)) {
        sorted = false;
        break;
      }
    }
  }
  expect(sorted).to.be.true;
}


export function cyIsSubFeatureEnabled(config: MinaNode, feature: FeatureType, subFeature: string, globalConfig: any): boolean {
  const features = getFeaturesConfig(config, globalConfig);
  return features[feature] && features[feature].includes(subFeature);
}

function getFeaturesConfig(config: MinaNode, globalConfig: any): FeaturesConfig {
  return config?.features || globalConfig?.features;
}
