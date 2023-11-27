import { Provider } from 'react-redux';

import { ActiveDeals } from '@components/ActiveDeals';
import { Charts } from '@components/Charts';
import { Header } from '@components/Header';
import { PotentialDeals } from '@components/PotentialDeals';

import { store } from '@app/store';
import { Statistic } from '@components/Statistic';

function App() {
  return (
    <Provider store={store}>
      <section className='bg-mainBackground'>
        <Header />
        <main className="max-w-wrapper m-auto px-12 py-10 text-lightText font-inter">
          <Statistic />
          <Charts />
          <PotentialDeals />
          <ActiveDeals />
        </main>
      </section>
    </Provider>
  );
}

export default App;
