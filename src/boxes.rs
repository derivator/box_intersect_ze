//! Boxes of various types and dimensions that can be checked for intersection
/// Trait for a `DIM`-dimensional box with bounds of type `Num`. More precisely, the
/// cartesian product of `DIM` half-open intervals.
/// You probably want to use one of the box types below instead of implementing this yourself.
pub trait BBox: Copy {
    const DIM: usize;
    type Num: Copy + PartialOrd;

    /// Returns the low boundary of this box in dimension `dim`.
    fn lo(&self, dim: usize) -> Self::Num;

    /// Returns the high boundary of this box in dimension `dim`.
    fn hi(&self, dim: usize) -> Self::Num;

    /// Returns `true` if the projection of this box in dimension `dim` contains `point`.
    fn contains_in(&self, dim: usize, point: Self::Num) -> bool {
        self.lo(dim) <= point && point < self.hi(dim)
    }

    /// Returns `true` if the projection of this box in dimension `dim` intersects [`lo`, `hi`)
    fn intersects_in(&self, dim: usize, lo: Self::Num, hi: Self::Num) -> bool {
        self.lo(dim) < hi && lo < self.hi(dim)
    }

    // Returns `true` if the box intersects the given other box.
    fn intersects(&self, other: &Self) -> bool {
        for dim in 0..Self::DIM {
            if !self.intersects_in(dim, other.lo(dim), other.hi(dim)) {
                return false;
            }
        }
        return true;
    }
}

/// A generic `N`-dimensional box with bounds of type `B`
#[derive(Clone, Copy, Debug)]
pub struct BoxND<B, const N: usize> {
    min: [B; N],
    max: [B; N],
}

impl<B: Copy, const N: usize> BoxND<B, N> {
    /// Creates a new box. The low and high boundaries of the box
    /// will be `min` and `max`, indexed by dimension.
    pub fn new(min: [B; N], max: [B; N]) -> Self {
        Self { min, max }
    }
}

impl<B, const N: usize> BBox for BoxND<B, N>
where
    B: Copy + PartialOrd,
{
    const DIM: usize = N;
    type Num = B;

    fn lo(&self, dim: usize) -> Self::Num {
        self.min[dim]
    }

    fn hi(&self, dim: usize) -> Self::Num {
        self.max[dim]
    }
}

/// A 2-dimensional box with generic bounds of type `B`
pub type Box2D<B> = BoxND<B, 2>;
/// A 2-dimensional box with bounds of type `f32`
pub type Box2Df32 = Box2D<f32>;
/// A 2-dimensional box with bounds of type `f64`
pub type Box2Df64 = Box2D<f64>;

/// A 3-dimensional box with generic bounds of type `B`
pub type Box3D<B> = BoxND<B, 3>;
/// A 3-dimensional box with bounds of type `f32`
pub type Box3Df32 = Box3D<f32>;
/// A 3-dimensional box with bounds of type `f64`
pub type Box3Df64 = Box3D<f64>;

#[test]
fn intersect() {
    let box0 = Box3Df32::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let box1 = Box3Df32::new([5.0, 5.0, 5.0], [15.0, 15.0, 15.0]);
    let box2 = Box3Df32::new([10.0, 10.0, 10.0], [20.0, 20.0, 20.0]); //touches tip of box0
    let box3 = Box3Df32::new([0.0, 0.0, 50.0], [20.0, 20.0, 60.0]); //intersects all except in dimension 2

    assert!(box0.intersects(&box0));

    assert!(box0.intersects(&box1));
    assert!(box1.intersects(&box0));

    assert!(!box0.intersects(&box2));
    assert!(!box2.intersects(&box0));

    assert!(box1.intersects(&box2));
    assert!(box2.intersects(&box1));

    assert!(!box0.intersects(&box3));
    assert!(!box3.intersects(&box0));
}
