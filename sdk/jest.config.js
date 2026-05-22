module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: ['**/tests/**/*.test.ts'],
  moduleNameMapper: {
    '^\\.\\./\\.\\./wasm/web/pkg/cloakmesh_wasm$': '<rootDir>/tests/mocks/cloakmesh_wasm.ts',
  },
};
