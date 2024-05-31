import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  globalConfig: {
    features: {
      'block-production': ['won-slots'],
    },
  },
  configs: [
    {
      name: 'Producer 11010',
      url: 'http://localhost:11010',
    },
    {
      name: 'Producer 11012',
      url: 'http://localhost:11012',
    },
    {
      name: 'Producer 11014',
      url: 'http://localhost:11014',
    },
  ],
};

