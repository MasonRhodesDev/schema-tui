use schema_tui::{ConfigSchema, SchemaTUI, OptionResolver, Theme};
use std::collections::HashMap;
use anyhow::Result;

fn main() -> Result<()> {
    let schema_json = std::fs::read_to_string("examples/dynamic_options_schema.json")?;
    let schema: ConfigSchema = serde_json::from_str(&schema_json)?;
    
    let initial_values = HashMap::new();
    let option_resolver = OptionResolver::new();
    let theme = Theme::default();
    
    let mut app = SchemaTUI::new(
        schema,
        initial_values,
        option_resolver,
        theme,
        None,
    );
    
    app.on_change(|key, value| {
        eprintln!("Changed: {} = {:?}", key, value);
    });
    
    app.run()?;
    
    Ok(())
}
