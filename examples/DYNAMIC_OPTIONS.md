# Dynamic Script Parameters for Option Sources

This example demonstrates the dynamic script parameters feature, which allows option sources to depend on other field values and receive them as parameters.

## Overview

The `depends_on` field in script-based option sources enables cascading/dependent dropdowns. When a dependency field changes, the script is re-executed with updated parameters, and the dropdown options are refreshed automatically.

## How It Works

### Schema Definition

```json
{
  "id": "language",
  "type": "enum",
  "options_source": {
    "type": "static",
    "values": ["en", "es", "fr"]
  },
  "default": "en"
},
{
  "id": "preview_model",
  "type": "enum",
  "options_source": {
    "type": "script",
    "command": "bash examples/scripts/list-models.sh preview ${voice_config.language}",
    "depends_on": ["voice_config.language"],
    "cache_duration": 300
  }
}
```

### Variable Substitution

Variables in the format `${section.field}` are substituted with current field values before script execution:
- `${voice_config.language}` → `"en"` (if language field is set to "en")
- Multiple variables can be used in a single command
- Missing values are replaced with empty strings

### Cache Behavior

- Cache key = `command_template + substituted_values`
- Each unique parameter combination is cached separately
- Example: `list-models.sh preview en` and `list-models.sh preview es` have separate cache entries
- Cache respects `cache_duration` setting (in seconds)

### Dependency Tracking

When a field value changes:
1. System scans all enum fields for `depends_on` references to the changed field
2. Invalidates cached widgets for dependent fields
3. Next time dependent field is activated, script re-runs with new values
4. Dropdown options update automatically

## Running the Example

```bash
cargo run --example dynamic_options
```

### Expected Behavior

1. Start with Language = "en"
2. Navigate to "Preview Model" field and press Enter
3. See English models: `whisper-en-tiny`, `whisper-en-base`, `whisper-en-small`
4. Exit the field (ESC)
5. Change Language to "es"
6. Navigate back to "Preview Model" and press Enter
7. See Spanish models: `whisper-es-tiny`, `whisper-es-base`

The model list updates automatically based on the selected language!

## Script Requirements

Scripts must:
- Accept parameters via command-line arguments or environment variables
- Output JSON array of strings: `["option1", "option2", "option3"]`
- Exit with status code 0 on success
- Handle missing/empty parameters gracefully

### Example Script

See `examples/scripts/list-models.sh`:

```bash
#!/bin/bash
MODEL_TYPE="$1"
LANGUAGE="$2"

case "$LANGUAGE" in
  "en")
    echo '["whisper-en-tiny", "whisper-en-base", "whisper-en-small"]'
    ;;
  "es")
    echo '["whisper-es-tiny", "whisper-es-base"]'
    ;;
  *)
    echo '[]'
    ;;
esac
```

## Use Cases

### Cascading Configuration

```json
{
  "id": "cloud_provider",
  "options_source": {"type": "static", "values": ["aws", "gcp", "azure"]}
},
{
  "id": "region",
  "options_source": {
    "type": "script",
    "command": "cloud-tool list-regions ${config.cloud_provider}",
    "depends_on": ["config.cloud_provider"]
  }
},
{
  "id": "instance_type",
  "options_source": {
    "type": "script",
    "command": "cloud-tool list-instances ${config.cloud_provider} ${config.region}",
    "depends_on": ["config.cloud_provider", "config.region"]
  }
}
```

### Filtered Lists

```json
{
  "id": "file_type",
  "options_source": {"type": "static", "values": ["image", "video", "document"]}
},
{
  "id": "file",
  "options_source": {
    "type": "script",
    "command": "find ~/media -type f -name '*.${config.file_type}'",
    "depends_on": ["config.file_type"]
  }
}
```

### API-Based Options

```json
{
  "id": "api_endpoint",
  "type": "string"
},
{
  "id": "resource",
  "options_source": {
    "type": "script",
    "command": "curl -s ${config.api_endpoint}/resources | jq -r '.[].name'",
    "depends_on": ["config.api_endpoint"],
    "cache_duration": 60
  }
}
```

## Error Handling

- **Missing dependency value**: Empty string substituted, script should handle gracefully
- **Script failure**: Dropdown shows empty options list, no error thrown to user
- **Invalid JSON output**: Dropdown shows empty options list
- **Script timeout**: Not implemented (uses default system timeout)

## Performance Considerations

- **Initial load**: All static options load immediately, script-based options load on first access
- **Cache hit**: Instant (no script execution)
- **Cache miss**: Script execution time (typically < 100ms for simple scripts)
- **Dependency change**: Widget invalidated, script re-runs on next activation
- **Multiple dependencies**: Script only runs once when any dependency changes

## Backward Compatibility

The `depends_on` field is optional. Existing schemas without it continue to work:

```json
{
  "type": "script",
  "command": "ls ~/.config"
}
```

This behaves exactly as before (static command, no parameter substitution).
