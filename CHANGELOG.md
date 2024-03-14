# Changelog

## [0.6.0] - 2024-03-14

- Add FromStr impl for simplified_ledger::Ledger (thanks to Tim Bates)
- Apply parsed Transaction metadata to Postings in simplified_ledger::Ledger (thanks to Tim Bates)
- Rename SimplificationError to Error and wrap ledger_parser::ParseError into it (thanks to Tim Bates)
- Remove zero-balance accounts from Balance when adding transactions (thanks to Tim Bates)

## [0.5.0] - 2024-03-05

- Update ledger-parser to 6

## [0.4.1] - 2022-02-19

### Added

- Simplified ledger - handle currency exchanges

## [0.4.0] - 2022-02-06

### Changed

- Prices API change - allow initializing with multiple ledgers

## [0.3.0] - 2022-01-25

### Added

- Calculate omitted amounts (thanks to Tim Bates)

## [0.2.0] - 2021-01-22

### Changed

- Update ledger-parser to 5

## [0.1.0] - 2021-01-04

### Added

- Initial version
