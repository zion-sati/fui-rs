import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default defineConfig(
  {
    ignores: [
      '**/node_modules/**',
      '**/public/**',
      '**/target/**',
      '**/test-results/**',
      '**/v2/core/tests/integration/report/**',
      '**/v2/create-fui-as-app/build/**',
      '**/v2/create-fui-as-app/dist/**',
      '**/v2/create-fui-as-app/templates/**',
      '**/v2/create-fui-rs-app/build/**',
      '**/v2/create-fui-rs-app/dist/**',
      '**/v2/create-fui-rs-app/templates/**',
      '**/v2/create-fui-rs-app/eslint.config.ts',
      '**/v2/fui-as/build/**',
      '**/v2/fui-as/demo/**',
      '**/v2/fui-as/src/**',
      '**/v2/fui-as/tests/fixtures/smoke/app.ts',
      '**/v2/fui-as/tests/fixtures/smoke/workers/**',
      '**/v2/fui-as/tests/unit/**',
    ],
  },
  {
    files: ['v2/**/*.ts'],
    extends: [
      js.configs.recommended,
      ...tseslint.configs.strictTypeChecked,
      ...tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-import-type-side-effects': 'error',
      '@typescript-eslint/no-invalid-void-type': ['error', {
        allowAsThisParameter: true,
        allowInGenericTypeArguments: true,
      }],
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_',
      }],
    },
  },
);
