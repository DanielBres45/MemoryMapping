use core::fmt;
use std::{cmp::Ordering, ops::{Add, Mul, Sub}};


#[derive(Clone, Copy, Hash, Debug)]
pub enum Offset {
    Neg(usize),
    Pos(usize)
}


impl fmt::Display for Offset
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Offset::Neg(value) => write!(f, "-{}", value),
            Offset::Pos(value) => write!(f, "+{}", value),
        }
    }
}

impl PartialEq for Offset
{
    fn eq(&self, other: &Self) -> bool
    {
        let norm_self: Self = self.normalized();
        let norm_other: Self = other.normalized();
        match (norm_self, norm_other)
        {
            (Offset::Pos(lhs), Offset::Pos(rhs)) | (Offset::Neg(lhs), Offset::Neg(rhs)) =>
            {
                lhs == rhs
            }
            _ =>
            {
                false
            }
        }
    }
}

impl PartialOrd for Offset
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_normalized: Offset = self.normalized();
        let other_normalized: Offset = other.normalized();

        if self_normalized.less_then(&other_normalized)
        {
            return Some(Ordering::Less);
        }

        if self_normalized == other_normalized
        {
            return Some(Ordering::Equal);
        }
        else 
        {
            return Some(Ordering::Greater)    
        }
    }
}

impl Mul<i8> for Offset{
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        match (self, Offset::from(rhs))
        {
            (Offset::Pos(lhs), Offset::Pos(rhs)) | 
            (Offset::Neg(lhs), Offset::Neg(rhs)) =>
            {
                Offset::Pos(lhs * rhs)
            },
            (Offset::Pos(lhs), Offset::Neg(rhs)) |
            (Offset::Neg(lhs), Offset::Pos(rhs)) =>
            {
                Offset::Neg(lhs * rhs)
            }
        }
    }
}

impl Mul<usize> for Offset{
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output{
        match self {
            Offset::Pos(p_val) => Offset::Pos(p_val * rhs),
            Offset::Neg(n_val) => Offset::Neg(n_val * rhs)
        }
    }
}

///Floating point multiplication with rounding. 
/// i.e 5 * 0.5 = 2.5 = ceiling(2.5) = 3
/// 5 * 0.45 = 2.25 = floor(2.25)

impl Mul<f32> for Offset{
    type Output = Option<Self>;

    fn mul(self, rhs: f32) -> Self::Output {
        if rhs.is_infinite() || rhs.is_nan()
        {
            return None;
        }

        match self
        {
            Offset::Pos(p_val) =>
            {
                let f_val = f32::round((p_val as f32) * rhs);
                let u_val =  usize::try_from((f_val as i32).abs()).unwrap();

                if rhs < 0.0
                {
                    Some(Offset::Neg(u_val))
                }
                else {
                    Some(Offset::Pos(u_val))
                }
            },
            Offset::Neg(n_val) =>
            {
                let f_val = f32::round((n_val as f32) * rhs);
                let u_val =  usize::try_from((f_val as i32).abs()).unwrap();

                if rhs < 0.0
                {
                    Some(Offset::Pos(u_val))
                }
                else 
                {
                    Some(Offset::Neg(u_val))    
                }
            }
        }
    }
}

impl Mul for Offset{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs)
        {
            (Offset::Pos(lhs), Offset::Pos(rhs)) | 
            (Offset::Neg(lhs), Offset::Neg(rhs)) =>
            {
                Offset::Pos(lhs * rhs)
            },
            (Offset::Pos(lhs), Offset::Neg(rhs)) |
            (Offset::Neg(lhs), Offset::Pos(rhs)) =>
            {
                Offset::Neg(lhs * rhs)
            }
        }
    }
}

impl Sub for Offset
{
    type Output = Self;

    fn sub(self, other: Self) -> Self
    {
        self + other.flip()
    }
}

impl Sub<Offset> for usize
{
    type Output = Offset;

    fn sub(self, rhs: Offset) -> Self::Output {
        rhs.flip() + self
    }
}

impl Add<Offset> for usize{
    type Output = Option<Self>;

    fn add(self, other: Offset) -> Option<Self>
    {
        match other
        {
            Offset::Pos(p_val) => Some(p_val + self),
            Offset::Neg(n_val) => self.checked_sub(n_val)
        }
    }
}

impl Add<usize> for Offset{
    type Output = Self;

    fn add(self, other: usize) -> Self
    {
        match self
        {
            Offset::Pos(lhs) => 
            {
                Offset::Pos(lhs + other)
            },
            Offset::Neg(lhs) =>
            {
                Offset::subtract_size(other, lhs)
            }
        }
    }
}

impl Add for Offset
{
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        let self_norm: Self = self.normalized();
        let other_norm: Self = other.normalized();

        match (self_norm, other_norm) {
            (Offset::Pos(lhs), Offset::Pos(rhs)) => Offset::Pos(lhs + rhs),

            (Offset::Neg(lhs), Offset::Neg(rhs)) => Offset::Neg(lhs + rhs),

            (Offset::Pos(lhs), Offset::Neg(rhs)) | (Offset::Neg(rhs), Offset::Pos(lhs)) => {
                Offset::subtract_size(lhs, rhs)
            }
        }
    }

}



impl From<i8> for Offset{
    fn from(value: i8) -> Self {
        if value < 0
        {
            Offset::Neg(i8::abs(value) as usize)
        }
        else {
            Offset::Pos(value as usize)
        }
    }
}

impl From<isize> for Offset{
    fn from(value: isize) -> Self {
        if value < 0
        {
            Offset::Neg(isize::unsigned_abs(value))
        }
        else 
        {
            Offset::Pos(value as usize)    
        }
    }
}

impl Offset
{
    pub fn abs(self) -> Self{
        match self{
            Offset::Pos(_) => self,
            Offset::Neg(neg_val) => Offset::Pos(neg_val)
        }
    }

    pub fn sqrt(self) -> f32{
        match self
        {
            Offset::Pos(val) => f32::sqrt(val as f32),
            Offset::Neg(_) => f32::NAN
        }
    }

    // Helper function to simplify subtraction logic
    #[inline]
    fn subtract_size(lhs: usize, rhs: usize) -> Offset {
        if lhs >= rhs {
            Offset::Pos(lhs - rhs)
        } else {
            Offset::Neg(rhs - lhs)
        }
    }

    pub fn normalized(&self) -> Offset
    {
        match &self
        {
            Offset::Pos(val) | Offset::Neg(val) => 
            {
                if *val == 0
                {
                    return Offset::Pos(0)
                }
            }
        }

        return self.clone()
    }

    pub fn less_then(&self, other: &Self) -> bool
    {
        let norm_self: Self = self.normalized();
        let norm_other: Self = other.normalized();
        match (norm_self, norm_other)
        {
            (Offset::Neg(_lhs), Offset::Pos(_rhs)) => true,
            (Offset::Neg(lhs), Offset::Neg(rhs)) => lhs > rhs,
            (Offset::Pos(lhs), Offset::Pos(rhs)) => lhs < rhs,
            (Offset::Pos(_lhs), Offset::Neg(_rhs)) => false
        }
    }

    pub fn zero() -> Self
    {
        Offset::Pos(0)
    }

    pub fn new(val: isize) -> Self
    {
        match val > 0
        {
            true => Offset::Pos(val as usize),
            false => Offset::Neg(val.abs() as usize)
        }
    }
    pub fn flip(&self) -> Offset
    {
        match self
        {
            Offset::Pos(val) => return Offset::Neg(*val),
            Offset::Neg(val) => return Offset::Pos(*val)
        }
    }

    pub fn range(&self, other: &Self) -> Option<Vec<Offset>>
    {
        let mut range: Vec<Offset> = Vec::new();
        match (self, other)
        {
            //other must be > self
            (Offset::Pos(_lhs), Offset::Neg(_rhs)) =>
            {
                None
            },
            (Offset::Pos(lhs), Offset::Pos(rhs)) =>
            {
                for i in *lhs..*rhs
                {
                    range.push(Offset::Pos(i));
                }

                if range.len() == 0
                {
                    None
                }
                else 
                {
                    Some(range)    
                }
            },
            (Offset::Neg(lhs), Offset::Pos(rhs)) =>
            {
                for i in 0..(*lhs + 1)
                {
                    range.push(Offset::Neg(i));
                }

                range.reverse();

                for i in 1..*rhs
                {
                    range.push(Offset::Pos(i));
                }

                if range.len() == 0
                {
                    None
                }
                else 
                {
                    Some(range)    
                }
            },
            (Offset::Neg(lhs), Offset::Neg(rhs)) =>
            {
                for i in *lhs..*rhs
                {
                    range.push(Offset::Pos(i));
                }

                if range.len() == 0
                {
                    None
                }
                else 
                {
                    Some(range)    
                }
            }
        } 
    }
}

