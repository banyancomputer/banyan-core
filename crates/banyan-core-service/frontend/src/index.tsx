import React from 'react';
import ReactDOM from 'react-dom/client';
import { Provider } from 'react-redux';

import { store } from './app/store';
import { ModalProvider } from './app/contexts/modals';
import App from './app/App';

import './index.css';

const root = ReactDOM.createRoot(
    document.getElementById('root') as HTMLElement
);

root.render(
    <Provider store={store}>
        <ModalProvider>
            <App />
        </ModalProvider>
    </Provider>
);
