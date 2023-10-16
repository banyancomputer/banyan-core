import React from 'react'
import { BucketBillingInfo } from '..'
import { convertFileSize } from '@/utils/storage'
import { useIntl } from 'react-intl'

export const Bucket: React.FC<{ billingInfo: BucketBillingInfo }> = ({ billingInfo }) => {
  const { messages } = useIntl();
  return (
    <div className='overflow-hidden  border-1 border-table-border rounded-lg'>
      <table className='w-full text-xs text-gray-600'>
        <thead className='bg-table-headBackground p-3'>
          <tr className='p-3 text-xxs border-b-1 font-medium border-table-border'>
            <td className='p-3'>{`${messages.name}`}</td>
            <td className='p-3'>{`${messages.dataQty}`}</td>
            <td className='p-3'>{`${messages.service}`}</td>
            <td className='p-3'>{`${messages.billiedAt}`}</td>
            <td className='p-3'>{`${messages.yourCost}`}</td>
          </tr>
        </thead>
        <tbody className=''>
          {
            billingInfo.data.map((data, index) =>
              <tr className='border-b-1 border-table-border'>
                {!index &&
                  <td
                    rowSpan={billingInfo.data.length}
                    className='p-3 border-r-1 border-table-border'
                  >
                    {billingInfo.name}
                  </td>
                }
                <td className='px-3 py-4'>{convertFileSize(data.memoryAmount)}</td>
                <td className='px-3 py-4'>{data.serviceName}</td>
                <td className='px-3 py-4'>{data.billiedAt}</td>
                <td className='px-3 py-4 text-sm text-gray-900 font-semibold'>${`${data.cost}`}</td>
              </tr>
            )
          }
        </tbody>
        <tfoot className='bg-table-headBackground'>
          <tr>
            <td
              colSpan={5}
              className='p-3 border-t-1 border-table-border text-xxs'
            >
              <div className='flex justify-end items-center gap-8'>
                {`${messages.billTotal}`}
                <span className='text-lg text-gray-900 font-semibold'>
                  $1000
                </span>
              </div>
            </td>
          </tr>
        </tfoot>
      </table>
    </div>
  )
}
