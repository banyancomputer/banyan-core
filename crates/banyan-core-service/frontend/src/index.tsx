import React from 'react';
import ReactDOM from 'react-dom/client';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';

import { store } from '@app/store';
import { ModalProvider } from '@contexts/modals';
import App from '@app/App';

import './index.css';

const root = ReactDOM.createRoot(
    document.getElementById('root') as HTMLElement
);

root.render(
    <Provider store={store}>
        <ModalProvider>
            <BrowserRouter basename="/" >
                <App />
            </BrowserRouter>
        </ModalProvider>
    </Provider>
);
