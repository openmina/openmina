/**
 * This configuration is used for the staging-webnode environment.
 */

export default {
  production: true,
  canAddNodes: false,
  showWebNodeLandingPage: true,
  showLeaderboard: true,
  hidePeersPill: true,
  hideTxPill: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
      'mempool': [],
      'state': ['actions'],
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
    heartbeats: true,
  },
  // sentry: {
  //   dsn: 'https://69aba72a6290383494290cf285ab13b3@o4508216158584832.ingest.de.sentry.io/4508216160616528',
  //   tracingOrigins: ['https://www.openmina.com', 'webnode-gtm-test.firebaseapp.com', 'webnode-gtm-test.firebasestorage.app'],
  // },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
