import { Clock } from '@static/images'

export const Actions = () => {
    return (
        <div className='flex flex-col w-32 rounded-lg overflow-hidden bg-contextMenuBackground'>
            <div className='flex items-center gap-2 px-2.5 py-3 transition-all hover:bg-contextMenuHoverackground'>
                <Clock />
                Details
            </div>
            <div className='flex items-center gap-2 px-2.5 py-3 transition-all hover:bg-contextMenuHoverackground'>
                <Clock />
                Download
            </div>
            <div className='flex items-center gap-2 px-2.5 py-3 transition-all hover:bg-contextMenuHoverackground'>
                <Clock />
                Cancel Deal
            </div>
        </div>
    )
}
