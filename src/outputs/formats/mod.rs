//! Predefined implementations of [FanFormat](crate::FanFormat), [FanBuilder](crate::FanBuilder), 
//! [ListFormat](crate::ListFormat), and [ListBuilder](crate::ListBuilder)

mod generic_fans;
pub(crate) use generic_fans::GenericFans;
mod generic_list;
pub(crate) use generic_list::GenericList;
mod indexed_fan;
pub use indexed_fan::IndexedFanFormat;
mod delimited_fan;
pub use delimited_fan::DelimitedFanFormat;
mod deindexed_fan;
pub use deindexed_fan::DeindexedFanFormat;
mod fan_to_list;
pub use fan_to_list::FanToListFormat;
mod indexed_list;
pub use indexed_list::IndexedListFormat;
mod deindexed_list;
pub use deindexed_list::DeindexedListFormat;
mod reverse_fan;
pub use reverse_fan::ReverseFanFormat;