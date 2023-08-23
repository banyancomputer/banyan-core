import React, { ReactElement } from 'react';
import {
    AiOutlineFile, AiOutlineFileExcel, AiOutlineFileGif,
    AiOutlineFileImage, AiOutlineFileJpg, AiOutlineFilePdf,
    AiOutlineFilePpt, AiOutlineFileText, AiOutlineFileWord,
    AiOutlineFileZip,
} from 'react-icons/ai';

export const FileIcon: React.FC<{ fileName: string; className?: string }> = ({ fileName, className }) => {
    const fileTypeMapper: Record<string, ReactElement> = {
        'txt': <AiOutlineFileText size="24px" fill="#4A5578" />,
        'pdf': <AiOutlineFilePdf size="24px" fill="#4A5578" />,
        'doc': <AiOutlineFileWord size="24px" fill="#4A5578" />,
        'docx': <AiOutlineFileWord size="24px" fill="#4A5578" />,
        'ppt': <AiOutlineFilePpt size="24px" fill="#4A5578" />,
        'pptx': <AiOutlineFilePpt size="24px" fill="#4A5578" />,
        'xls': <AiOutlineFileExcel size="24px" fill="#4A5578" />,
        'xlsx': <AiOutlineFileExcel size="24px" fill="#4A5578" />,
        'jpg': <AiOutlineFileJpg size="24px" fill="#4A5578" />,
        'jpeg': <AiOutlineFileJpg size="24px" fill="#4A5578" />,
        'png': <AiOutlineFileImage size="24px" fill="#4A5578" />,
        'gif': <AiOutlineFileGif size="24px" fill="#4A5578" />,
        'zip': <AiOutlineFileZip size="24px" fill="#4A5578" />,
        'rar': <AiOutlineFileZip size="24px" fill="#4A5578" />,
    };

    return (
        <div className={className}>{fileTypeMapper[fileName.split('.')[1]] || <AiOutlineFile size="24px" />}</div>
    );
};
