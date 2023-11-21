import { useEffect, useRef, useState } from 'react';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';

import { popupClickHandler } from '@app/utils/clickHandlers';

describe('clickHandlers', () => {
    const TestComponent: React.FC = () => {
        const [isVisible, setIsVisible] = useState(true);
        const testRef = useRef<HTMLDivElement | null>(null);

        useEffect(() => {
            const listener = popupClickHandler(testRef.current!, setIsVisible);
            document.addEventListener('click', listener);

            return () => document.removeEventListener('click', listener);
        }, [testRef]);

        return <div data-testid="outside">
            {isVisible &&
                <div data-testid="target" ref={testRef}>
                    <span data-testid="inside"></span>
                </div>
            }
        </div>
    };

    test('Click outside of ref should trigger state', () => {
        render(<TestComponent />);
        expect(screen.queryByTestId('target')).toBeInTheDocument();
        act(() => userEvent.click(screen.getByTestId('outside')));
        expect(screen.queryByTestId('target')).toBeNull();
    });

    test('Click inside of ref should not trigger state', () => {
        render(<TestComponent />);
        expect(screen.queryByTestId('target')).toBeInTheDocument();
        act(() => userEvent.click(screen.getByTestId('inside')));
        expect(screen.queryByTestId('target')).toBeInTheDocument();
    });
})