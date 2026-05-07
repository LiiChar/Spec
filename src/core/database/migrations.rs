use rusqlite_migration::{M, Migrations};

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        r#"
        ALTER TABLE jobs ADD COLUMN tags TEXT;
        "#,
    ),
    M::up(
        r#"
        ALTER TABLE goals ADD COLUMN tags TEXT;
        "#,
    ),
];

pub const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);