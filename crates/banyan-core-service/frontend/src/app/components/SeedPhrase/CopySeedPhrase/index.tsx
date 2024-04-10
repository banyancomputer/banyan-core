import React from 'react';
import { SeedPhraseInput } from '@components/common/SeedPhraseInput';
import { Copy } from '@static/images/common';
import { PrimaryButton } from '../../common/PrimaryButton';
import { ToastNotifications } from '@/app/utils/toastNotifications';

const CopySeedPhrase: React.FC<{
    seedPhrase: string[],
    onChange: React.Dispatch<React.SetStateAction<string[]>>,
    setStep: React.Dispatch<React.SetStateAction<'copying' | 'confirmation'>>,
}> = ({
    seedPhrase,
    onChange,
    setStep
}) => {

        const copy = (event: React.MouseEvent<HTMLButtonElement>) => {
            event.preventDefault();
            event.stopPropagation();
            navigator.clipboard?.writeText(seedPhrase.join(' '));
            ToastNotifications.notify('Seed phrase copied');
        };
        return (
            <div className="flex flex-col gap-4">
                <div className="flex flex-col gap-2 items-center">
                    <h2 className="text-lg font-semibold">Write down your Seed Phrase</h2>
                    <p className="text-xs">Copy this 12-word Seed Phrase and save it in a place that you trust.</p>
                </div>
                <SeedPhraseInput seedPhrase={seedPhrase} />
                <button
                    className="flex items-center mx-auto gap-2 p-2  text-button-primary font-semibold text-sm"
                    onClick={copy}
                >
                    <Copy />
                    Copy to clipboard
                </button>
                <PrimaryButton
                    text="Next"
                    action={() => setStep('confirmation')}
                />
                <p className='text-xs text-center'>You won’t be able recover this after you leave this screen.</p>
            </div>
        );
    };

export default CopySeedPhrase;
