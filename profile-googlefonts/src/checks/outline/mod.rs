mod alignment_miss;
use std::ops::Sub;

pub use alignment_miss::alignment_miss;
mod direction;
pub use direction::direction;
mod jaggy_segments;
pub use jaggy_segments::jaggy_segments;
mod semi_vertical;
pub use semi_vertical::semi_vertical;
mod short_segments;
pub use short_segments::short_segments;

pub(crate) fn close_but_not_on<T>(expected: T, actual: T, epsilon: T) -> bool
where
    T: Sub<Output = T> + PartialOrd + Copy + num_traits::sign::Signed,
{
    (actual - expected).abs() <= epsilon && actual != expected
}
