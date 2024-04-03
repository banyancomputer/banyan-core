import React, { useEffect, useState } from 'react'
import CopySeedPhrase from '../components/SeedPhrase/CopySeedPhrase';
import { ConfirmSeedPhrase } from '../components/SeedPhrase/ConfirmSeedPhrase';

const SeedPhrase = () => {
  const [seedPhrase, setSeedPhrase] = useState<string[]>(new Array(12).fill('test'));
  const [step, setStep] = useState<'copying' | 'confirmation'>('copying');

  useEffect(() => {
    (async () => {
      /**TODO: implement getting seed phrase. */
    })();
  }, []);

  return (
    <div className="flex flex-col gap-4">
      {step === 'copying' ?
        <CopySeedPhrase
          seedPhrase={seedPhrase}
          onChange={setSeedPhrase}
          setStep={setStep}
        />
        :
        <ConfirmSeedPhrase
          seedPhrase={seedPhrase}
        />
      }
    </div>
  );
};

export default SeedPhrase;
