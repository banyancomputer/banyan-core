import React from 'react';
import { CgSpinnerTwoAlt } from "react-icons/cg";

export const Loader: React.FC<{
    spinnerSize: string;
    containerHeight?: string;
}> = ({ spinnerSize, containerHeight = 'unset' }) =>
        <div className="w-full h-full flex justify-center items-center" style={{ height: containerHeight }}>
            <div className="animate-spin" style={{ height: spinnerSize, width: spinnerSize }}>
                <CgSpinnerTwoAlt size="50px" />
            </div>
        </div>;

