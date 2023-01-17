module.exports = {
  env: {
    browser: true,
    es2021: true
  },
  plugins: ["solid"],
  extends: ['standard-with-typescript', "eslint:recommended", "plugin:solid/typescript", 'prettier'],
  overrides: [
  ],
  ignorePatterns: ["src/pkg/*"],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    project: "./tsconfig.json"
  },
  rules: {
    quotes: ["error", "double"]
  }
}
