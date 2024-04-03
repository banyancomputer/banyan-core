import React, { ChangeEvent, useState } from 'react'

export const SeedPhraseInput: React.FC<{
  seedPhrase: string[],
  onChange?: React.Dispatch<React.SetStateAction<string[]>>,
}> = ({
  onChange,
  seedPhrase
}) => {

    const handleChange = (event: ChangeEvent<HTMLInputElement>, index: number) => {
      onChange && onChange(prev => prev.map((word, wordIndex) => wordIndex === index ? event.target.value : word));
    };

    return (
      <div className={`grid grid-cols-[repeat(auto-fill,minmax(126px,1fr))] gap-4`}>
        {seedPhrase.map((word, index) =>
          <div className="flex items-center justify-end gap-4 text-sm font-medium">
            <span>{index + 1}.</span>
            <input
              className={`w-24 px-4 py-2 text-center rounded-md border-1 border-border-regular bg-mainBackground ${onChange && index % 4 ? 'text-gray-500 pointer-events-none' : ''}`}
              type="text"
              value={word}
              onChange={event => handleChange(event, index)}
            />
          </div>
        )}
      </div>
    )
  }
