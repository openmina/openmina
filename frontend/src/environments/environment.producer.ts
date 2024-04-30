import { MinaEnv } from '@shared/types/core/environment/mina-env.type';

export const environment: Readonly<MinaEnv> = {
  production: true,
  configs: [
    {
      name: 'Producer',
      url: 'http://65.109.105.40:3000',
      features: {
        'block-production': [],
      },
    },
  ],
};
