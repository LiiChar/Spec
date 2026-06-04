# Migration Guide: Adding Color to window_activity

## What Changed

### Schema
- **V2 Migration**: Adds `color TEXT DEFAULT 'bg-secondary/20'` column to `window_activity` table
- **Automatically applied** on database initialization
- **Safe for existing databases** - adds column with default value

### Models
```rust
// WindowModel now includes:
pub struct WindowModel {
    // ... existing fields ...
    pub color: String,  // NEW FIELD
}
```

### Database API
```rust
// New methods:
db.update_window_color(window_id, "bg-blue/20")?;
db.get_window_color(window_id)?;
```

### Repositories
```rust
// WindowRepository methods:
WindowRepository::update_color(&conn, id, color)?;
WindowRepository::get_color(&conn, id)?;
```

### Schema Constants
```rust
use crate::core::database::schema::window_activity;

// Type-safe column reference:
window_activity::COL_COLOR  // = "color"
```

## For Developers

### When You Create a WindowModel
```rust
let window = WindowModel {
    // ... other fields ...
    color: "bg-secondary/20".to_string(),  // Provide this now
};
db.insert_window(&window)?;
```

### When You Update a Window
```rust
// To change color:
db.update_window_color(window.id.unwrap(), "bg-red/20")?;
```

### When You Query Windows
```rust
let (window, tags) = db.get_windows_by_process("chrome.exe")?
    .first()
    .unwrap();

println!("Color: {}", window.color);
```

## Backward Compatibility

✅ **Fully Compatible**
- Existing databases are automatically migrated
- Old windows get default color `'bg-secondary/20'`
- No data loss
- Migration runs only once per database

## Benefits

| Before | After |
|--------|-------|
| Windows had no color field | Windows can have custom colors |
| Manual query strings for colors | Type-safe database operations |
| No schema versioning | Versioned migrations (V1, V2, ...) |
| Hard to maintain | Modular, organized code |
| 57KB database.rs file | 6 focused repository files |

## Testing the Migration

```bash
# Database is automatically migrated on creation
cargo run  # Migration runs automatically

# Check schema version was created:
# SELECT * FROM schema_version;
# Should show:
# version | applied_at | name
# 1       | ...        | initial_schema
# 2       | ...        | add_window_color
```

## Adding More Fields in Future

Following this pattern makes it easy to add new fields:

1. **Create migration** in `migrations.rs`:
   ```rust
   if current_version < 3 {
       apply_v3_add_new_field(conn)?;
       // ...
   }
   ```

2. **Update schema** in `schema.rs`:
   ```rust
   pub const COL_NEW_FIELD: &str = "new_field";
   ```

3. **Update model** in `window.rs`:
   ```rust
   pub struct WindowModel {
       // ...
       pub new_field: SomeType,
   }
   ```

4. **Add repository method** in `window_repo.rs`:
   ```rust
   pub fn get_new_field(conn: &Connection, id: i64) -> Result<SomeType> {
       // ...
   }
   ```

## FAQ

**Q: Will my database break?**
A: No, migrations are backwards compatible and only run once.

**Q: How do I check the schema version?**
A: Query the `schema_version` table:
```sql
SELECT * FROM schema_version;
```

**Q: What if I miss applying a migration?**
A: It's automatic - migrations run on Database::new()

**Q: Can I rollback a migration?**
A: Not automatically. For now, manually restore from backup if needed.

**Q: How do I add another migration?**
A: Follow the pattern in migrations.rs - create v3, v4, etc.
