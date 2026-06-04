# Database Reorganization Documentation

## Overview

The database layer has been reorganized into a modular, type-safe architecture. Instead of all operations in a single 57KB `database.rs` file, we now have:

- **Migrations** - Versioned schema changes
- **Schema** - Centralized column/table name constants
- **Repositories** - Domain-specific data access patterns
- **Database** - High-level public API

## Architecture

```
src/core/database/
├── database.rs           # Public API, delegates to repositories
├── migrations.rs         # Schema versioning (V1, V2, ...)
├── schema.rs             # Table/column definitions
└── repositories/
    ├── window_repo.rs    # Window activity operations
    ├── tag_repo.rs       # Tag management
    ├── job_repo.rs       # Job records
    ├── goal_repo.rs      # Goal records
    └── event_repo.rs     # Event records
```

## Key Features

### 1. Type-Safe Repositories

Each repository provides a clean API for its domain:

```rust
// Window operations
WindowRepository::get_all_windows(&conn)?;
WindowRepository::insert(&conn, &window)?;
WindowRepository::update_color(&conn, window_id, "bg-blue/20")?;

// Tag operations
TagRepository::get_by_name(&conn, "Work")?;
TagRepository::add_to_window(&conn, tag_id, "chrome.exe")?;

// Job operations
JobRepository::insert(&conn, &job)?;
JobRepository::update(&conn, &job)?;
```

### 2. Schema Definitions

All column names are constants, preventing typos:

```rust
use crate::core::database::schema::window_activity;

// Instead of magic strings
SELECT window_activity::COL_COLOR FROM window_activity::TABLE
// Type-safe constants:
// window_activity::COL_ID
// window_activity::COL_TITLE
// window_activity::COL_COLOR (NEW!)
```

### 3. Versioned Migrations

Migrations are tracked and applied sequentially:

```rust
// V1: Initial schema (tables created)
// V2: Add color to window_activity (NEW!)
//     - Adds schema_version table
//     - Tracks which migrations have been applied
//     - Safe to run multiple times
```

## Example: Adding a New Column

### Before (Old Approach)
1. Manual ALTER TABLE in code
2. Use `ensure_column()` helper
3. No version tracking
4. Easy to forget in new environments

### After (New Approach)
1. Create new migration in `migrations.rs`:
   ```rust
   if current_version < 3 {
       apply_v3_add_new_field(conn)?;
       conn.execute(
           "INSERT INTO schema_version (version, name) VALUES (3, 'add_new_field')",
           [],
       )?;
   }
   ```

2. Add schema constants:
   ```rust
   // In schema.rs
   pub const COL_NEW_FIELD: &str = "new_field";
   ```

3. Use in repository:
   ```rust
   repository_method(conn) -> Result<()> {
       conn.execute(
           &format!("UPDATE {} SET {} = ?1", TABLE, COL_NEW_FIELD),
           params,
       )?;
       Ok(())
   }
   ```

## Migration: Adding `color` to window_activity

**What was done:**
1. Created migration V2 that adds `color TEXT DEFAULT 'bg-secondary/20'`
2. Updated `WindowModel` with `color: String` field
3. Added `WindowRepository::update_color()` method
4. Updated `WindowRepository::insert()` to save color
5. Schema constants in `window_activity::COL_COLOR`

**Result:**
- Windows now track their display color
- Safe migration from old databases
- New databases created with color support

## Using the Database

### Basic Window Operations

```rust
// Get all windows
let windows = db.get_windows()?;

// Update window color
db.update_window_color(window_id, "bg-red/20")?;

// Get window color
let color = db.get_window_color(window_id)?;
```

### Working with Tags

```rust
// Create tag (idempotent)
let tag = db.ensure_tag("Work", "#3b82f6")?;

// Add to window
db.add_tag_to_window(tag.id.unwrap(), "chrome.exe".into())?;

// Check if applied
let has_tag = db.has_tag_for_window("chrome.exe", tag_id)?;
```

### Working with Jobs

```rust
// Create job
let mut job = JobModel::default();
job.name = "Project X".into();
job.color = "bg-blue/20".into();
db.save_job(&job)?;

// Update job
db.update_job(&job)?;

// Delete job
db.delete_job(job.id.unwrap())?;
```

## Benefits of New Architecture

| Aspect | Before | After |
|--------|--------|-------|
| **File Size** | 57KB single file | Modular ~8KB files |
| **Code Reuse** | Mixed concerns | Clear separation |
| **Adding Fields** | Manual, error-prone | Versioned migrations |
| **Testability** | Hard to test | Repository-based, easier to mock |
| **Discoverability** | Long file to search | Focused modules |
| **Type Safety** | Magic strings for columns | Schema constants |

## Testing

Each repository includes tests:

```bash
cargo test --lib core::database::repositories
```

## Future Enhancements

1. Add transaction support in `DbContext`
2. Create query builder for complex queries
3. Add connection pooling for multi-threaded access
4. Implement soft deletes with timestamps
5. Add audit logging for changes

## Notes

- Migrations are idempotent (safe to run multiple times)
- Color field defaults to `'bg-secondary/20'` for consistency
- Schema version table persists migration history
- All repositories accept `&Connection` for flexibility
