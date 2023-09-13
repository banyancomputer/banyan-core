import React, { useEffect } from 'react';

import getServerSideProps from '@/utils/session';
import { NextPageWithLayout } from './page';

export { getServerSideProps };

const Settings: NextPageWithLayout = () => {

    return (
        <div>Settings</div>
    )
}


export default Settings;
