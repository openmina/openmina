/**
 * This configuration is used for the webnode environment.
 *
 * NOTE: When modifying environment configuration files, update the documentation at:
 * website/docs/developers/frontend/environment-configuration.mdx
 */

export default {
  production: true,
  canAddNodes: false,
  showWebNodeLandingPage: true,
  globalConfig: {
    features: {
      dashboard: [],
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
