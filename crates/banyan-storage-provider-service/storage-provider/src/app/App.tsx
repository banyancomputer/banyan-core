import { ActiveDeals } from '@components/ActiveDeals';
import { Charts } from '@components/Charts';
import { Header } from '@components/Header';
import { PotentialDeal, PotentialDeals } from '@components/PotentialDeals';
import { ServiceDetails } from '@components/ServiceDetails';

function App() {
  return (
    <section className='bg-mainBackground'>
      <Header />
      <main className="max-w-wrapper m-auto px-12 py-10 text-lightText font-inter">
        <h1 className='mb-20  text-80 font-light font-boogy tracking-tighter'>Storage Provider Management Dashboard</h1>
        <Charts />
        <ServiceDetails />
        <PotentialDeals />
        <ActiveDeals />
      </main>
    </section>
  );
}

export default App;
