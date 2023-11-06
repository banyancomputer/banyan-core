import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './app/App';

import './index.css';
import { ModalProvider } from '@app/contexts/modals';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);
root.render(
  <ModalProvider>
    <App />
  </ModalProvider>
);
