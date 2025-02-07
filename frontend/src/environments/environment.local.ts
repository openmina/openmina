/**
 * This file is used for local starting of the app without any development intentions.
 */

import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: false,
  identifier: 'Local FE',
  canAddNodes: true,
  globalConfig: {
    features: {
      dashboard: [],
      nodes: ['overview', 'live', 'bootstrap'],
      state: ['actions'],
      snarks: ['scan-state', 'work-pool'],
      mempool: [],
      'block-production': ['won-slots'],
    },
    firebase: {
      apiKey: 'AIzaSyBZzFsHjIbQVbBP0N-KkUsEvHRVU_wwd7g',
      authDomain: 'webnode-gtm-test.firebaseapp.com',
      projectId: 'webnode-gtm-test',
      storageBucket: 'webnode-gtm-test.firebasestorage.app',
      messagingSenderId: '1016673359357',
      appId: '1:1016673359357:web:bbd2cbf3f031756aec7594',
      measurementId: 'G-ENDBL923XT',
    },
  },
  configs: [
    {
      name: 'Local rust node',
      url: 'http://127.0.0.1:3000',
    },
  ],
};

