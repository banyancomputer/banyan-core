import React from 'react';

import { ErrorBanner } from '@components/common/ErrorBanner';
import { Header } from '@components/common/Header';
import { Navigation } from '@components/common/Navigation';
import { BetaBanner } from '../components/common/BetaBanner';

export const CommonLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <section className="flex flex-grow">
      <Navigation />
      <section className="flex-grow flex flex-col h-screen overflow-y-scroll">
        <Header />
        <BetaBanner />
        <ErrorBanner />
        {children}
      </section>
    </section>
  )
}
