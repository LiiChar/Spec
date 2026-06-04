/// Repository modules for organized database access
/// Each repository handles a specific domain entity

pub mod window_repo;
pub mod tag_repo;
pub mod job_repo;
pub mod goal_repo;
pub mod event_repo;

pub use window_repo::WindowRepository;
pub use tag_repo::TagRepository;
pub use job_repo::JobRepository;
pub use goal_repo::GoalRepository;
pub use event_repo::EventRepository;
