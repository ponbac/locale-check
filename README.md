# Ramilang

Command-line interface (CLI) tool to handle and validate translations in `RamiCustomerPortal`.

## Features

- **Detect Duplicate Keys:** Finds duplicate keys within the translation files.
- **Validate Key Compatibility:** Ensures that the keys in different translation files match.
- **Find Missing and Empty Keys:** Detects any missing or empty keys.
- **Sort Translation Keys:** Option to sort keys in the translation files.
- **Check Unused Keys:** Detects keys that are not being used in the codebase (`ts` and `tsx` files).
- **Detect Usage of Invalid Keys:** Detects usage of keys that does not exist (`ts` and `tsx` files).
- **Custom Ignore List:** Ability to ignore certain keys from the unused keys check. Useful for keys that are used in a non-standard way, making static analysis hard.

## Usage from customer portal `turborepo` root

### Run checks, ignoring unused entries present in `.keyignore`

```bash
pnpm check-translations
```

### Run checks, ignoring unused entries present in `.keyignore` and sort keys alphabetically

```bash
pnpm check-translations:sort
```

## Manual usage with pnpm

### Running latest version from `npm`

```bash
pnpx ramilang@latest --en-file ./shared/translations/en.json --sv-file ./shared/translations/sv.json
```

### with an ignore list

```bash
pnpx ramilang@latest --en-file ./shared/translations/en.json --sv-file ./shared/translations/sv.json --ignore-file ./shared/translations/.keyignore
```

### with alphabetical sorting on keys

```bash
pnpx ramilang@latest --sort --en-file ./shared/translations/en.json --sv-file ./shared/translations/sv.json
```

## Arguments

- `--en-file`: Path to English translation file.
- `--sv-file`: Path to Swedish translation file.
- `--root-dir`: Root directory to search from (default is current directory).
- `--ignore-file`: Path to file with line separated translation keys to exclude from unused check.
- `--sort`: Sort keys alphabetically in translation files.
