module.exports = {
    injectGlobals: true,
    preset: "ts-jest",
    testEnvironment: 'jsdom',
    moduleDirectories: ["node_modules", "src"],
    moduleNameMapper: {
        '@app/(.*)': '<rootDir>/src/app/$1',
        '@components/(.*)': '<rootDir>/src/app/components/$1',
        '@utils/(.*)': '<rootDir>/src/app/utils/$1',
        '^@/(.*)$': '<rootDir>/src/$1',
        '^@static/(.*)$': '<rootDir>/src/app/static/$1',
        '\\.(css|less|sass|scss)$': 'identity-obj-proxy'
    },
    roots: ['<rootDir>'],
    transform: {
        '^.+\\.(ts|tsx)?$': 'ts-jest',
    },
    testRegex: '(/tests/.*|(\\.|/)(test|spec))\\.(tsx|ts)?$',
    moduleFileExtensions: ['ts', 'js', 'tsx', 'jsx', 'json'],
    collectCoverage: true,
    clearMocks: true,
    coverageDirectory: "coverage",
};