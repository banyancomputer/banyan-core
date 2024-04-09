import { Suspense } from 'react';
import { Provider } from 'react-redux';
import { BrowserRouter } from 'react-router-dom';

import { store } from '@app/store';
import { Routes } from './routes';
import { Header } from '@components/common/Header';

function App() {
  return (
    <section className='bg-mainBackground'>
      <Provider store={store}>
        <BrowserRouter basename="/" >
          <Header />
          <main className="max-w-wrapper m-auto px-12 py-10 text-lightText font-inter">
            <Suspense>
              <Routes />
            </Suspense>
          </main>
        </BrowserRouter>
      </Provider>
    </section>
  );
}

export default App;
