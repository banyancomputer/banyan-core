import React from 'react';
import { CgSpinnerTwoAlt } from 'react-icons/cg';

export const Loader: React.FC<{
    spinnerSize: string;
    containerHeight?: string;
    className?: string;
}> = ({ spinnerSize, containerHeight = 'unset', className = '' }) =>
        <div className="w-full h-full flex justify-center items-center" style={{ height: containerHeight }}>
            <div className={`animate-spin ${className}`} style={{ height: spinnerSize, width: spinnerSize }}>
                <CgSpinnerTwoAlt size={spinnerSize} />
            </div>
        </div>;