# Database Architecture - Quick Start

## Files Structure

```
src/core/database/
├── database.rs           # Public API (150 lines)
│   ├── with_database()   # Access DB from UI
│   ├── get_windows()
│   ├── insert_window()
│   ├── update_window_color()  # NEW
│   └── ... other methods
│
├── migrations.rs         # Schema versioning (160 lines)
│   ├── V1: Initial schema
│   └── V2: Add color to window_activity
│
├── schema.rs             # Type-safe column names (95 lines)
│   ├── window_activity::COL_COLOR
│   ├── window_activity::COL_TITLE
│   └── ... other tables
│
└── repositories/         # Domain-specific access
    ├── mod.rs
    ├── window_repo.rs    # Window operations (250 lines)
    ├── tag_repo.rs       # Tag operations (195 lines)
    ├── job_repo.rs       # Job operations (175 lines)
    ├── goal_repo.rs      # Goal operations (145 lines)
    └── event_repo.rs     # Event operations (85 lines)
```

## Total Lines of Code

- **Before**: 1,472 lines (all in `database.rs`)
- **After**: ~1,300 lines (modular, organized)
- **Reduction**: Clean separation of concerns

## Quick Usage

### Get Windows
```rust
let windows = db.get_windows()?;
for (window, tags) in windows {
    println!("Process: {}, Color: {}", window.process_name, window.color);
}
```

### Update Window Color
```rust
db.update_window_color(window_id, "bg-red/20")?;
```

### Work with Tags
```rust
let tag = db.ensure_tag("Work", "#3b82f6")?;
db.add_tag_to_window_if_missing("Work", "chrome.exe".into())?;
```

### Work with Jobs
```rust
let job = JobModel::default();
db.save_job(&job)?;
db.update_job(&job)?;
```

## New Feature: Window Colors

**Problem**: Windows had no visual distinction
**Solution**: Added `color` field to `window_activity` table

```rust
pub struct WindowModel {
    // ... existing fields ...
    pub color: String,  // e.g., "bg-blue/20", "bg-red/20"
}
```

**Migration**: V2 automatically adds column with default value

## Database Schema (V2)

```
window_activity (main table)
├── id                 (PK)
├── hwnd              (window handle)
├── title             (window title)
├── process_name      (app name)
├── color             ← NEW!
├── timestamp         (when tracked)
├── duration          (how long)
└── ... (20+ more columns)

events
├── id                (PK)
├── window_activity_id (FK)
├── event_type        (idle/keyboard/mouse)
└── timestamp         (when)

jobs
├── id                (PK)
├── name              (job name)
├── color             (job color)
└── ... (7 more fields)

goals
├── id                (PK)
├── name              (goal name)
└── ... (8 more fields)

tag
├── id                (PK)
├── name              (tag name)
└── color             (tag color)

tag_to_window (many-to-many)
├── tag_id            (FK)
└── process_name      (e.g., "chrome.exe")

schema_version (NEW!)
├── version           (PK)
├── applied_at        (timestamp)
└── name              (migration name)
```

## API Reference

### Window Operations
```rust
db.get_windows() -> Result<Vec<(WindowModel, Vec<TagModel>)>>
db.get_windows_by_process(name) -> Result<Vec<...>>
db.insert_window(window) -> Result<i64>
db.delete_window(name) -> Result<()>
db.update_window_color(id, color) -> Result<()>  // NEW
db.get_window_color(id) -> Result<Option<String>>  // NEW
```

### Tag Operations
```rust
db.get_tags() -> Result<Vec<TagModel>>
db.get_tag_by_name(name) -> Result<Option<TagModel>>
db.ensure_tag(name, color) -> Result<TagModel>
db.get_window_tag(process) -> Result<Vec<TagModel>>
db.add_tag_to_window(tag_id, process) -> Result<i64>
db.add_tag_to_window_if_missing(tag_name, process) -> Result<()>
db.merge_tags(tags) -> Result<usize>
```

### Job Operations
```rust
db.get_jobs() -> Result<Vec<JobModel>>
db.save_job(job) -> Result<i64>
db.update_job(job) -> Result<()>
db.delete_job(id) -> Result<()>
db.get_jobs_for_day(start, end) -> Result<Vec<JobModel>>
```

### Goal Operations
```rust
db.get_goals() -> Result<Vec<GoalModel>>
db.insert_goal(goal) -> Result<i64>
db.update_goal(goal) -> Result<()>
db.delete_goal(id) -> Result<()>
```

### Event Operations
```rust
db.insert_event(event) -> Result<()>
db.insert_events(events) -> Result<()>
```

### Analytics
```rust
db.get_top_processes() -> Result<Vec<(String, i64)>>
db.get_total_time_by_process() -> Result<Vec<(String, i64)>>
db.cleanup_old(older_than_ts) -> Result<()>
```

## Key Improvements

| Aspect | Improvement |
|--------|-------------|
| **Modularity** | Separate repo files for each entity |
| **Type Safety** | Schema constants prevent typos |
| **Maintainability** | Focused files are easier to understand |
| **Extensibility** | Add new fields via versioned migrations |
| **Documentation** | Each repo has clear purpose |
| **Testing** | Smaller units are easier to test |
| **Color Support** | Windows now have customizable colors |

## Next Steps

1. **Test the migration**: Run `cargo build` - should work automatically
2. **Use new color API**: See examples.rs for usage patterns
3. **Add more fields**: Follow migration pattern for future changes
4. **Create UI**: Use color field in UI components

## Migration Checklist

- [x] Create migration system (V1, V2)
- [x] Add schema constants
- [x] Create repositories (window, tag, job, goal, event)
- [x] Add color field to WindowModel
- [x] Implement WindowRepository::update_color()
- [x] Implement WindowRepository::get_color()
- [x] Refactor Database to use repositories
- [x] Document migration
- [x] Create usage examples
- [x] Add FAQ and troubleshooting

## Questions?

See:
- `DATABASE_REFACTORING.md` - Architecture overview
- `MIGRATION_GUIDE.md` - Step-by-step migration info
- `examples.rs` - Code examples
- Repository files - Detailed implementation
