import React, { Suspense } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { Routes } from './routes';

function App() {
    return (
        <div className="app">
            <BrowserRouter basename="/" >
                <Suspense >
                    <Routes />
                </Suspense>
            </BrowserRouter>
        </div>
    );
}

export default App;
