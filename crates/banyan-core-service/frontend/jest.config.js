module.exports = {
    injectGlobals: true,
    testEnvironment: 'jsdom',
    "presets": [
        "@babel/preset-env"
    ],
    moduleDirectories: ["node_modules", "src"],
   
    moduleNameMapper: {
        '@app/(.*)': '<rootDir>/src/app/$1',
        '@components/(.*)': '<rootDir>/src/app/components/$1',
        '@utils/(.*)': '<rootDir>/src/app/utils/$1',
        '@pages/(.*)': '<rootDir>/src/app/pages/$1',
        '@store/(.*)': '<rootDir>/src/app/store/$1',
        '^@/(.*)$': '<rootDir>/src/$1',
        '^@static/(.*)$': '<rootDir>/src/app/static/$1',
        '\\.(css|less|sass|scss)$': 'identity-obj-proxy',
    },
    roots: ['<rootDir>'],
    testRegex: '(/tests/.*|(\\.|/)(test|spec))\\.(tsx|ts)?$',
    coveragePathIgnorePatterns: ['/node_modules/', '.*.svg', '.*.png'],
    transformIgnorePatterns: [
        'node_modules/(?!((jest-)?react-native(-.*)?|@react-native(-community)?)/)',
      ],

    moduleFileExtensions: ['ts', 'js', 'tsx', 'jsx', 'json'],
    collectCoverage: false,
    clearMocks: true,
    coverageDirectory: "coverage",
};