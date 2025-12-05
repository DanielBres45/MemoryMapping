use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::ops::Range;
use crate::memory_span2d::MemSpan2D;

///half open (inclusive lower_bound, exclusive upper_bound) span of memory indexes
/// not quite a Range<usize> as that is a destructive range for iteration

#[derive(Clone, Debug, Copy)]
pub struct MemSpan
{
    pub min: usize,
    pub count: usize
}

impl IntoIterator for MemSpan
{
    type Item = usize;
    type IntoIter = std::ops::Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.min..self.upper_bound()
    }
}

impl From<Range<usize>> for MemSpan
{
    fn from(range: Range<usize>) -> Self {
        MemSpan::new_range(range)
    }
}

impl Eq for MemSpan {}

impl Ord for MemSpan
{
    fn cmp(&self, other: &Self) -> Ordering {
        let lower_cmp = self.min.cmp(&other.min);
        if lower_cmp != Ordering::Equal
        {
            return lower_cmp;
        }

        return self.count.cmp(&other.count);
    }
}

impl PartialOrd for MemSpan
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl MemSpan
{
    pub fn lower_bound(&self) -> usize
    {
        self.min
    }

    pub fn upper_bound(&self) -> usize
    {
        self.min + self.count
    }

    pub fn max(&self) -> Option<usize>
    {
        if self.count == 0 {
            return None;
        }

        Some(self.min + self.count - 1)
    }

    pub fn max_value(&self) -> Option<usize>
    {
        self.max()
    }

    pub fn len(&self) -> usize
    {
        self.count
    }

    pub fn min_max(min: usize, max: usize) -> Option<Self>
    {
        MemSpan::lower_bound_upper_bound(min, max + 1)
    }

    pub fn lower_bound_upper_bound(lower: usize, upper: usize) -> Option<Self>
    {
        if lower > upper {
            return None;
        }

        Some(MemSpan{ min: lower, count: upper - lower })
    }
    #[inline]
    pub fn new_range(range: Range<usize>) -> Self
    {
        MemSpan{min: range.start, count: range.count()}
    }

    // Wrapping versions
    #[inline]
    pub fn shift_up_wrapping(&self, shift: usize) -> Self {
        MemSpan {
            min: self.min.wrapping_add(shift),
            count: self.count,
        }
    }

    #[inline]
    pub fn shift_down_wrapping(&self, shift: usize) -> Self {
        MemSpan {
            min: self.min.wrapping_sub(shift),
            count: self.count,
        }
    }

    pub fn shift_wrapping(&self, shift: isize) -> Self {
        if shift < 0 {
            self.shift_down_wrapping(-shift as usize)
        } else if shift > 0 {
            self.shift_up_wrapping(shift as usize)
        } else {
            self.clone()
        }
    }

    // Checked versions 
    #[inline]
    pub fn shift_up_checked(&self, shift: usize) -> Option<Self> {
        let min = self.min.checked_add(shift)?;

        if min.checked_add(self.count).is_none() {
            return None;
        }

        Some(MemSpan {min, count: self.count})
    }

    #[inline]
    pub fn shift_down_checked(&self, shift: usize) -> Option<Self> {
        let start = self.min.checked_sub(shift)?;
        Some(MemSpan {
            min: start,
            count: self.count,
        })
    }

    pub fn shift_checked(&self, shift: isize) -> Option<Self> {
        if shift < 0 {
            self.shift_down_checked(-shift as usize)
        } else if shift > 0 {
            self.shift_up_checked(shift as usize)
        } else {
            Some(self.clone())
        }
    }

    // Max shift wrapping versions
    #[inline]
    pub fn shift_max_up_wrapping(&self, shift: usize) -> Self {
        MemSpan{ min: self.min, count: self.count.wrapping_add(shift)}
    }

    #[inline]
    pub fn shift_max_down_wrapping(&self, shift: usize) -> Self {
        MemSpan{min: self.min, count: self.count.wrapping_sub(shift)}
    }

    pub fn shift_max_wrapping(&self, shift: isize) -> Self {
        if shift < 0 {
            self.shift_max_down_wrapping(-shift as usize)
        } else if shift > 0 {
            self.shift_max_up_wrapping(shift as usize)
        } else {
            self.clone()
        }
    }

    // Max shift checked versions
    #[inline]
    pub fn shift_max_up_checked(&self, shift: usize) -> Option<Self> {
        let count = self.count.checked_add(shift)?;

        if self.min.checked_add(count).is_none() {
            return None;
        }


        Some(MemSpan{min: self.min, count})
    }

    #[inline]
    pub fn shift_max_down_checked(&self, shift: usize) -> Option<Self> {
        let count = self.count.checked_sub(shift)?;
        Some(MemSpan{min: self.min, count})
    }

    pub fn shift_max_checked(&self, shift: isize) -> Option<Self> {
        if shift < 0 {
            self.shift_max_down_checked(-shift as usize)
        } else if shift > 0 {
            self.shift_max_up_checked(shift as usize)
        } else {
            Some(self.clone())
        }
    }

    // Min shift wrapping versions
    #[inline]
    pub fn shift_min_up_wrapping(&self, shift: usize) -> Self {
        let min = self.min.wrapping_add(shift);
        let count = self.count.wrapping_sub(shift);
        MemSpan{min, count}
    }

    #[inline]
    pub fn shift_min_down_wrapping(&self, shift: usize) -> Self {
        MemSpan{min: self.min.wrapping_sub(shift), count: self.count}
    }

    pub fn shift_min_wrapping(&self, shift: isize) -> Self {
        if shift < 0 {
            self.shift_min_down_wrapping(-shift as usize)
        } else if shift > 0 {
            self.shift_min_up_wrapping(shift as usize)
        } else {
            self.clone()
        }
    }

    // Min shift checked versions
    #[inline]
    pub fn shift_min_up_checked(&self, shift: usize) -> Option<Self> {
        let min = self.min.checked_add(shift)?;
        let count: usize = self.count.checked_sub(shift)?;

        Some(MemSpan{min, count})
    }

    #[inline]
    pub fn shift_min_down_checked(&self, shift: usize) -> Option<Self> {
        let min = self.min.checked_sub(shift)?;
        let count: usize = self.count.checked_add(shift)?;
        Some(MemSpan{min, count})
    }

    pub fn shift_min_checked(&self, shift: isize) -> Option<Self> {
        if shift < 0 {
            self.shift_min_down_checked(-shift as usize)
        } else if shift > 0 {
            self.shift_min_up_checked(shift as usize)
        } else {
            Some(self.clone())
        }
    }

    #[inline]
    pub fn contains(&self, index: usize) -> bool
    {
        self.min <= index && index < self.count
    }

    #[inline]
    pub fn intersect(&self, other: &Self) -> Option<Self>
    {
        let this_max: usize = self.max()?;
        let other_max = other.max_value()?;
        Self::min_max(self.min.max(other.min), this_max.min(other_max))
    }
    pub fn overlaps(&self, other: &Self) -> bool
    {
        self.intersect(other).is_some_and(|s| s.len() > 0)
    }

    ///Check if the sorted list of MemSpans contain any overlap
    pub fn spans_have_overlap_sorted(spans: &Vec<MemSpan>) -> bool
    {
        for i in 0..spans.len() - 1
        {
            if let Some(max) = spans[i].max_value()
            {
                if spans[i+1].min < max
                {
                    return true;
                }
            }
        }

        false
    }
}

impl PartialEq for MemSpan
{
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.count == other.count
    }
}

impl Display for MemSpan
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{})", self.min, self.max().unwrap_or(self.min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ///5 cases
    ///A_min < B_min & A_max < B_Max -> A_min..B_max
    ///A_min == B_min && A_max == B_Max -> A_min..B_max
    ///A_min > B_min && A_max < B_Max -> A_min..A_max
    ///A_min < B_max && A_max > B_max -> A_min..B_max
    #[test]
    fn test_intersection()
    {
        let mut A = MemSpan::min_max(1, 100).unwrap();
        assert_eq!(100, MemSpan::max(&A).unwrap());
        let mut B = MemSpan::min_max(50, 150).unwrap();
        let mut intersect: MemSpan = A.intersect(&B).unwrap();
        assert_eq!(50, intersect.min);
        assert_eq!(100, MemSpan::max(&intersect).unwrap());

        A = MemSpan::min_max(1, 100).unwrap();
        B = MemSpan::min_max(1, 100).unwrap();
        intersect = A.intersect(&B).unwrap();
        assert_eq!(1, intersect.min);
        assert_eq!(100, intersect.max_value().unwrap());

        intersect = B.intersect(&A).unwrap();
        assert_eq!(1, intersect.min);
        assert_eq!(100, intersect.max_value().unwrap());

        A = MemSpan::min_max(50, 75).unwrap();
        B = MemSpan::min_max(1, 100).unwrap();
        intersect = A.intersect(&B).unwrap();
        assert_eq!(50, intersect.min);
        assert_eq!(75, MemSpan::max(&intersect).unwrap());


        intersect = B.intersect(&A).unwrap();
        assert_eq!(50, intersect.min);
        assert_eq!(75, intersect.max_value().unwrap());

        A = MemSpan::min_max(50, 100).unwrap();
        B = MemSpan::min_max(1, 75).unwrap();
        intersect = A.intersect(&B).unwrap();
        assert_eq!(50, intersect.min);
        assert_eq!(75, intersect.max_value().unwrap());

    }
}