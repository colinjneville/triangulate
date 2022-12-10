/// A type which can map its members of type `T` to other types
/// (e.g. `[usize; 2]` is [Mappable] over [usize] to produce `[u16; 2]`, `[u32; 2]`, etc.)
/// 
/// It is recommended to not import this trait directly since it has a blanket 
/// implementation which will conflict with other `map` functions.
pub trait Mappable<T> {
    /// The resulting type when `T` is mapped to `U`
    type Output<U>;
    /// Uses function `f` to map all associated `T` types to `U`
    fn map<F: Fn(T) -> U, U>(self, f: F) -> Self::Output<U>;
}

impl<T> Mappable<T> for T {
    type Output<U> = U;

    fn map<F: Fn(T) -> U, U>(self, f: F) -> Self::Output<U> {
        f(self)
    }
}

impl<T, const N: usize> Mappable<T> for [T; N] {
    type Output<U> = [U; N];

    fn map<F: Fn(T) -> U, U>(self, f: F) -> Self::Output<U>  {
        self.map(f)
    }
}

impl<T> Mappable<T> for (T, T) {
    type Output<U> = (U, U);

    fn map<F: Fn(T) -> U, U>(self, f: F) -> Self::Output<U>  {
        let (a, b) = self;
        (f(a), f(b))
    }
}

impl<T> Mappable<T> for (T, T, T) {
    type Output<U> = (U, U, U);

    fn map<F: Fn(T) -> U, U>(self, f: F) -> Self::Output<U>  {
        let (a, b, c) = self;
        (f(a), f(b), f(c))
    }
}

impl<T> Mappable<T> for Vec<T> {
    type Output<U> = Vec<U>;

    fn map<F: Fn(T) -> U, U>(self, f: F) -> Self::Output<U>  {
        Iterator::map(self.into_iter(), f).collect()
    }
}
