import React from 'react';

import { ErrorBanner } from '@components/common/ErrorBanner';
import { Header } from '@components/common/Header';
import { Navigation } from '@components/common/Navigation';

export const CommonLayout: React.FC<{ children: React.ReactNode }> = ({ children }) =>
  <section className="flex flex-col items-stretch flex-grow max-h-full overflow-auto">
    <div className="flex flex-grow">
      <Navigation />
      <section className="flex-grow flex flex-col h-full overflow-y-auto">
        <ErrorBanner />
        <Header />
        {children}
      </section>
    </div>
  </section>;
