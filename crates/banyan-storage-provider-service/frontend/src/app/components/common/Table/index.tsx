import React from 'react'

export const Table = () => {
    return (
        <div className='rounded-xl overflow-hidden border-1 border-tableBorder'>
            <table className="w-full">
                <thead className='bg-tableHead text-12'>
                    <tr>
                        <th className='p-3 text-justify border-1 border-t-0 border-l-0 border-tableBorder'>Storage Provider Identity</th>
                        <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Currently used storage</th>
                        <th className='p-3 text-justify border-1 border-t-0 border-tableBorder'>Avialable/ allocated storage</th>
                        <th className='p-3 text-justify border-1 border-t-0 border-r-0 border-tableBorder'>Start of billing period</th>
                    </tr>
                </thead>
                <tbody className='bg-tableBody text-14'>
                    <tr>
                        <td className='py-4 px-3 border-1 border-l-0 border-tableBorder'>Provider 1</td>
                        <td className='py-4 px-3 border-1 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-r-0 border-tableBorder'>05.10.2023</td>
                    </tr>
                    <tr>
                        <td className='py-4 px-3 border-1 border-l-0 border-tableBorder'>Provider 1</td>
                        <td className='py-4 px-3 border-1 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-r-0 border-tableBorder'>05.10.2023</td>
                    </tr>
                    <tr>
                        <td className='py-4 px-3 border-1 border-b-0 border-l-0 border-tableBorder'>Provider 1</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-r-0 border-tableBorder'>05.10.2023</td>
                    </tr>
                    <tr>
                        <td className='py-4 px-3 border-1 border-b-0 border-l-0 border-tableBorder'>Provider 1</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-r-0 border-tableBorder'>05.10.2023</td>
                    </tr>
                    <tr>
                        <td className='py-4 px-3 border-1 border-b-0 border-l-0 border-tableBorder'>Provider 1</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-tableBorder'>456 TiB</td>
                        <td className='py-4 px-3 border-1 border-b-0 border-r-0 border-tableBorder'>05.10.2023</td>
                    </tr>
                </tbody>
            </table>
        </div>
    )
}
