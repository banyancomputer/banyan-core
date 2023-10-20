import React, { ReactElement } from 'react';
import {
    AiOutlineFile, AiOutlineFileExcel, AiOutlineFileGif,
    AiOutlineFileImage, AiOutlineFileJpg, AiOutlineFilePdf,
    AiOutlineFilePpt, AiOutlineFileText, AiOutlineFileWord,
    AiOutlineFileZip,
} from 'react-icons/ai';
import { PiFolderNotchBold } from 'react-icons/pi';

export const FileIcon: React.FC<{ fileName: string; className?: string, size?: string }> = ({ fileName, className, size = '20px' }) => {
    const fileTypeMapper: Record<string, ReactElement> = {
        'txt': <AiOutlineFileText size={size} />,
        'pdf': <AiOutlineFilePdf size={size} />,
        'doc': <AiOutlineFileWord size={size} />,
        'docx': <AiOutlineFileWord size={size} />,
        'ppt': <AiOutlineFilePpt size={size} />,
        'pptx': <AiOutlineFilePpt size={size} />,
        'xls': <AiOutlineFileExcel size={size} />,
        'xlsx': <AiOutlineFileExcel size={size} />,
        'jpg': <AiOutlineFileJpg size={size} />,
        'jpeg': <AiOutlineFileJpg size={size} />,
        'png': <AiOutlineFileImage size={size} />,
        'gif': <AiOutlineFileGif size={size} />,
        'zip': <AiOutlineFileZip size={size} />,
        'rar': <AiOutlineFileZip size={size} />,
    };

    return (
        <div className={className}>
            {
                fileName.split('.')[1] ?
                    fileTypeMapper[fileName.split('.')[1]] || <AiOutlineFile size={size} />
                    :
                    <PiFolderNotchBold size={size} />
            }
        </div>
    );
};
