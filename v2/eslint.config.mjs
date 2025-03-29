import eslint from '@eslint/js';
import ts from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import security from 'eslint-plugin-security';
import prettier from 'eslint-config-prettier/flat';

export default ts.config(
    eslint.configs.recommended,
    reactHooks.configs['recommended-latest'],
    security.configs.recommended,
    ...ts.configs.recommendedTypeChecked,
    prettier,
    {
        languageOptions: {
            parserOptions: {
                projectService: true,
                project: 'tsconfig.json',
            },
        },
    },
    {
        rules: {
            'prefer-spread': 'off',
            'no-unused-vars': 'off',
            'no-return-await': 'off',
            'security/detect-object-injection': 'off',
            eqeqeq: 'error',
            'require-await': 'error',
            'no-console': 'error',
            '@typescript-eslint/no-explicit-any': 'off',
            '@typescript-eslint/no-non-null-assertion': 'off',
            '@typescript-eslint/explicit-member-accessibility': 'off',
            '@typescript-eslint/no-floating-promises': 'error',
            '@typescript-eslint/no-unnecessary-type-arguments': 'error',
            '@typescript-eslint/no-empty-interface': 'error',
            '@typescript-eslint/restrict-plus-operands': 'error',
            '@typescript-eslint/no-extra-non-null-assertion': 'error',
            '@typescript-eslint/prefer-nullish-coalescing': 'error',
            '@typescript-eslint/prefer-optional-chain': 'error',
            '@typescript-eslint/prefer-includes': 'error',
            '@typescript-eslint/prefer-for-of': 'error',
            '@typescript-eslint/prefer-string-starts-ends-with': 'error',
            '@typescript-eslint/prefer-readonly': 'error',
            '@typescript-eslint/prefer-ts-expect-error': 'error',
            '@typescript-eslint/no-non-null-asserted-optional-chain': 'error',
            '@typescript-eslint/await-thenable': 'error',
            '@typescript-eslint/no-unnecessary-boolean-literal-compare': 'error',
            '@typescript-eslint/switch-exhaustiveness-check': 'error',
            '@typescript-eslint/explicit-function-return-type': ['error', { allowExpressions: true }],
            '@typescript-eslint/ban-ts-comment': [
                'error',
                {
                    'ts-ignore': true,
                    'ts-nocheck': true,
                },
            ],
            '@typescript-eslint/naming-convention': [
                'error',
                {
                    selector: 'default',
                    format: ['camelCase', 'PascalCase', 'UPPER_CASE'],
                    leadingUnderscore: 'allow',
                    filter: {
                        regex: '^aria-label$',
                        match: false,
                    },
                },
            ],
            '@typescript-eslint/no-unused-vars': 'error',
            '@typescript-eslint/no-confusing-void-expression': 'error',
            '@typescript-eslint/non-nullable-type-assertion-style': 'error',
            '@typescript-eslint/return-await': ['error', 'in-try-catch'],
            '@typescript-eslint/consistent-type-imports': ['error', { fixStyle: 'inline-type-imports' }],
            '@typescript-eslint/no-invalid-void-type': 'error',
            '@typescript-eslint/prefer-as-const': 'error',
            '@typescript-eslint/consistent-indexed-object-style': 'error',
            '@typescript-eslint/no-base-to-string': 'error',
            '@typescript-eslint/no-unnecessary-condition': ['error', { allowConstantLoopConditions: true }],
            '@typescript-eslint/no-unsafe-enum-comparison': 'error',
            '@typescript-eslint/no-deprecated': 'error',
            'react-hooks/exhaustive-deps': 'error',
            'react-hooks/rules-of-hooks': 'error',
        },
    }
);
