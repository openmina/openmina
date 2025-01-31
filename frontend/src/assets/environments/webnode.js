/**
 * This configuration is used for the staging-webnode environment.
 */

export default {
  production: true,
  canAddNodes: false,
  showWebNodeLandingPage: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
      'mempool': [],
      'benchmarks': ['wallets'],
      'state': ['actions'],
    },
    firebase: {
      'projectId': 'openminawebnode',
      'appId': '1:120031499786:web:9af56c50ebce25c619f1f3',
      'storageBucket': 'openminawebnode.firebasestorage.app',
      'apiKey': 'AIzaSyBreMkb5-8ANb5zL6yWKgRAk9owbDS1g9s',
      'authDomain': 'openminawebnode.firebaseapp.com',
      'messagingSenderId': '120031499786',
      'measurementId': 'G-V0ZC81T9RQ',
    },
  },
  sentry: {
    dsn: 'https://69aba72a6290383494290cf285ab13b3@o4508216158584832.ingest.de.sentry.io/4508216160616528',
    tracingOrigins: ['https://www.openmina.com', 'openminawebnode.firebaseapp.com', 'openminawebnode.firebasestorage.app'],
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
