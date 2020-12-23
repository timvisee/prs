pub mod add;
#[cfg(any(unix, windows))]
pub mod alias;
pub mod clone;
#[cfg(feature = "clipboard")]
pub mod copy;
pub mod duplicate;
pub mod edit;
pub mod generate;
pub mod git;
pub mod housekeeping;
pub mod init;
pub mod internal;
pub mod list;
pub mod r#move;
pub mod recipients;
pub mod remove;
pub mod show;
pub mod sync;

// Re-export to cmd module
pub use self::add::CmdAdd;
#[cfg(any(unix, windows))]
pub use self::alias::CmdAlias;
pub use self::clone::CmdClone;
#[cfg(feature = "clipboard")]
pub use self::copy::CmdCopy;
pub use self::duplicate::CmdDuplicate;
pub use self::edit::CmdEdit;
pub use self::generate::CmdGenerate;
pub use self::git::CmdGit;
pub use self::housekeeping::CmdHousekeeping;
pub use self::init::CmdInit;
pub use self::internal::CmdInternal;
pub use self::list::CmdList;
pub use self::r#move::CmdMove;
pub use self::recipients::CmdRecipients;
pub use self::remove::CmdRemove;
pub use self::show::CmdShow;
pub use self::sync::CmdSync;
