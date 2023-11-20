import { BrowserObject, FileMetadata } from "@app/types/bucket";
import { sortByMetadataField, sortByName, sortByType, sortFiles } from "@app/utils/sort";


export class MockBrowserObject implements BrowserObject {
    [key: string]: string | BrowserObject[] | FileMetadata;
    constructor(
        public name: string,
        public type: 'file' | 'dir',
        public metadata: FileMetadata
    ) {}

    public files = [] as BrowserObject[];
};

let MOCK_RESPONSE: BrowserObject[];

beforeEach(() => {
    MOCK_RESPONSE = [
        new MockBrowserObject('downloads', 'dir', {created: '11221', modified: '56855', size: 10000223}),
        new MockBrowserObject('videos', 'dir', {created: '866774', modified: '78367', size: 18353}),
        new MockBrowserObject('kafka.jpg', 'file', {created: '65754', modified: '46827', size: 57245567}),
        new MockBrowserObject('photos', 'dir', {created: '23466', modified: '74354', size: 8256597}),
        new MockBrowserObject('portfolio.pdf', 'file', {created: '43424', modified: '79638', size: 535921}),
        new MockBrowserObject('documents', 'dir', {created: '74589', modified: '61956', size: 957352448}),
        new MockBrowserObject('trash', 'dir', {created: '34685', modified: '37459', size: 323456}),
        new MockBrowserObject('wedding.mp4', 'file', {created: '53974', modified: '55367', size: 364572326}),
        new MockBrowserObject('common', 'dir', {created: '68735', modified: '72857', size: 52835573}),
    ];
});

describe(
    'sorting',
    () => {
        test(
            'sortByType',
            () => {
                MOCK_RESPONSE.sort(sortByType);
                expect(MOCK_RESPONSE).toEqual([
                    new MockBrowserObject('downloads', 'dir', {created: '11221', modified: '56855', size: 10000223}),
                    new MockBrowserObject('videos', 'dir', {created: '866774', modified: '78367', size: 18353}),
                    new MockBrowserObject('photos', 'dir', {created: '23466', modified: '74354', size: 8256597}),
                    new MockBrowserObject('documents', 'dir', {created: '74589', modified: '61956', size: 957352448}),
                    new MockBrowserObject('trash', 'dir', {created: '34685', modified: '37459', size: 323456}),
                    new MockBrowserObject('common', 'dir', {created: '68735', modified: '72857', size: 52835573}),
                    new MockBrowserObject('kafka.jpg', 'file', {created: '65754', modified: '46827', size: 57245567}),
                    new MockBrowserObject('portfolio.pdf', 'file', {created: '43424', modified: '79638', size: 535921}),
                    new MockBrowserObject('wedding.mp4', 'file', {created: '53974', modified: '55367', size: 364572326}),
                ]);
            }
            );
            test(
                'sortByName',
                () => {
                    const expectedResult = [
                        new MockBrowserObject('common', 'dir', {created: '68735', modified: '72857', size: 52835573}),
                        new MockBrowserObject('documents', 'dir', {created: '74589', modified: '61956', size: 957352448}),
                        new MockBrowserObject('downloads', 'dir', {created: '11221', modified: '56855', size: 10000223}),
                        new MockBrowserObject('kafka.jpg', 'file', {created: '65754', modified: '46827', size: 57245567}),
                        new MockBrowserObject('photos', 'dir', {created: '23466', modified: '74354', size: 8256597}),
                        new MockBrowserObject('portfolio.pdf', 'file', {created: '43424', modified: '79638', size: 535921}),
                        new MockBrowserObject('trash', 'dir', {created: '34685', modified: '37459', size: 323456}),
                        new MockBrowserObject('videos', 'dir', {created: '866774', modified: '78367', size: 18353}),
                        new MockBrowserObject('wedding.mp4', 'file', {created: '53974', modified: '55367', size: 364572326}),
                    ];
                    MOCK_RESPONSE.sort((prev, next) => sortByName(prev, next, true));
                    expect(MOCK_RESPONSE).toEqual(expectedResult);
                    MOCK_RESPONSE.sort((prev, next) => sortByName(prev, next, false));
                    expect(MOCK_RESPONSE).toEqual(expectedResult.reverse());
                }
                );
                test(
                    'sortByMetadataField',
                    () => {
                    const expectedResult = [
                        new MockBrowserObject('downloads', 'dir', {created: '11221', modified: '56855', size: 10000223}),
                        new MockBrowserObject('photos', 'dir', {created: '23466', modified: '74354', size: 8256597}),
                        new MockBrowserObject('trash', 'dir', {created: '34685', modified: '37459', size: 323456}),
                        new MockBrowserObject('portfolio.pdf', 'file', {created: '43424', modified: '79638', size: 535921}),
                        new MockBrowserObject('wedding.mp4', 'file', {created: '53974', modified: '55367', size: 364572326}),
                        new MockBrowserObject('kafka.jpg', 'file', {created: '65754', modified: '46827', size: 57245567}),
                        new MockBrowserObject('common', 'dir', {created: '68735', modified: '72857', size: 52835573}),
                        new MockBrowserObject('documents', 'dir', {created: '74589', modified: '61956', size: 957352448}),
                        new MockBrowserObject('videos', 'dir', {created: '866774', modified: '78367', size: 18353}),
                    ];
                    MOCK_RESPONSE.sort((prev, next) => sortByMetadataField(prev, next, 'created', true));
                    expect(MOCK_RESPONSE).toEqual(expectedResult);
                    MOCK_RESPONSE.sort((prev, next) => sortByMetadataField(prev, next, 'created', false));
                    expect(MOCK_RESPONSE).toEqual(expectedResult.reverse());
            }
            );
            test(
                'sortFilesBySize',
                () => {
                const expectedResult = [
                    new MockBrowserObject('videos', 'dir', {created: '866774', modified: '78367', size: 18353}),
                    new MockBrowserObject('trash', 'dir', {created: '34685', modified: '37459', size: 323456}),
                    new MockBrowserObject('portfolio.pdf', 'file', {created: '43424', modified: '79638', size: 535921}),
                    new MockBrowserObject('photos', 'dir', {created: '23466', modified: '74354', size: 8256597}),
                    new MockBrowserObject('downloads', 'dir', {created: '11221', modified: '56855', size: 10000223}),
                    new MockBrowserObject('common', 'dir', {created: '68735', modified: '72857', size: 52835573}),
                    new MockBrowserObject('kafka.jpg', 'file', {created: '65754', modified: '46827', size: 57245567}),
                    new MockBrowserObject('wedding.mp4', 'file', {created: '53974', modified: '55367', size: 364572326}),
                    new MockBrowserObject('documents', 'dir', {created: '74589', modified: '61956', size: 957352448}),
                ];

                MOCK_RESPONSE.sort((prev, next) => sortFiles(prev, next, 'size', true));
                expect(MOCK_RESPONSE).toEqual(expectedResult);
                MOCK_RESPONSE.sort((prev, next) => sortFiles(prev, next, 'size', false));
                expect(MOCK_RESPONSE).toEqual(expectedResult.reverse());
            }
        );
            test(
                'sortFilesByName',
                () => {
                const expectedResult = [
                    new MockBrowserObject('common', 'dir', {created: '68735', modified: '72857', size: 52835573}),
                    new MockBrowserObject('documents', 'dir', {created: '74589', modified: '61956', size: 957352448}),
                    new MockBrowserObject('downloads', 'dir', {created: '11221', modified: '56855', size: 10000223}),
                    new MockBrowserObject('kafka.jpg', 'file', {created: '65754', modified: '46827', size: 57245567}),
                    new MockBrowserObject('photos', 'dir', {created: '23466', modified: '74354', size: 8256597}),
                    new MockBrowserObject('portfolio.pdf', 'file', {created: '43424', modified: '79638', size: 535921}),
                    new MockBrowserObject('trash', 'dir', {created: '34685', modified: '37459', size: 323456}),
                    new MockBrowserObject('videos', 'dir', {created: '866774', modified: '78367', size: 18353}),
                    new MockBrowserObject('wedding.mp4', 'file', {created: '53974', modified: '55367', size: 364572326}),
                ];

                MOCK_RESPONSE.sort((prev, next) => sortFiles(prev, next, 'name', true));
                expect(MOCK_RESPONSE).toEqual(expectedResult);
                MOCK_RESPONSE.sort((prev, next) => sortFiles(prev, next, 'name', false));
                expect(MOCK_RESPONSE).toEqual(expectedResult.reverse());
            }
        );
    }
);
