const buildPrettierCommand = (filenames) =>
  `prettier ${filenames.join(' ')} -w`;

export default {
  "programs/*/src/**/*.rs": "cargo fmt --",
  "tests/**/*.ts": [buildPrettierCommand],
}