/**
 * This configuration is used for the webnode environment.
 */

export default {
  production: true,
  canAddNodes: false,
  showWebNodeLandingPage: true,
  globalConfig: {
    features: {
      'dashboard': [],
      'block-production': ['won-slots'],
    },
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
