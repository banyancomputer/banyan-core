import React from 'react';
import ReactDOM from 'react-dom/client';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';

import App from '@app/App';

import { store } from '@app/store';

import './index.css';

const root = ReactDOM.createRoot(
    document.getElementById('root') as HTMLElement
);

root.render(
    <Provider store={store}>
            <BrowserRouter basename="/" future={{ v7_startTransition: true }}>
                <App />
            </BrowserRouter>
    </Provider>
);
