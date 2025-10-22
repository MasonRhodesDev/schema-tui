# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Core schema type system with JSON deserialization
- Schema parser and validator
- Dynamic option resolution (static, script, function, file_list)
- Option caching system with TTL
- Config store with nested field access
- TOML config loader with environment variable expansion
- Config saver with schema-driven comment generation
- Widget system:
  - TextInput with cursor navigation
  - Toggle for boolean values
  - Dropdown for enum selection
  - SearchableDropdown with fuzzy filtering
  - NumberInput with validation
- Theme system respecting terminal colors
- Comprehensive test coverage (schema parsing, config loading, widgets)
- Public API with builder pattern
- Environment variable expansion (~, $VAR, ${VAR})

### Changed
- Widgets use Color::Reset by default to respect user's terminal theme

## [0.1.0] - Initial Release

Built by Mason Rhodes
