import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './app/App';
import { Notifications } from '@components/common/Notifications';

import './index.css';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);
root.render(
  <>
    <Notifications />
    <App />
  </>
);
