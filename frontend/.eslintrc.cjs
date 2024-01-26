/* eslint-env node */
require("@rushstack/eslint-patch/modern-module-resolution");

module.exports = {
	root: true,
	extends: [
		"plugin:vue/vue3-essential",
		"eslint:recommended",
		"@vue/eslint-config-typescript",
		"@vue/eslint-config-prettier/skip-formatting",
		"prettier",
	],
	parserOptions: {
		ecmaVersion: "latest",
	},
	plugins: ["prettier"],
	rules: {
		"prettier/prettier": "error",
		"vue/no-reserved-component-names": "off",
		"@typescript-eslint/no-unused-vars": 'off'
	},
};
