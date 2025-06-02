pub struct Simple2DArray<const COLS: usize, T> {
    arr: Vec<T>,
}

impl<const COLS: usize, T> Simple2DArray<COLS, T>
where
    T: Clone + Copy + Default,
{
    #[inline]
    pub fn new(rows: usize) -> Self {
        let arr = vec![Default::default(); rows * COLS];
        Self { arr }
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> T {
        self.arr[row * COLS + col]
    }

    #[inline]
    pub const fn rows(&self) -> usize {
        self.arr.len() / COLS
    }

    /// - - - - -
    /// x x x x x
    /// - - - - -
    #[inline]
    pub fn fill(&mut self, row: usize, value: T) {
        self.fill_rows(row, 1, value);
    }

    /// - - - - -
    /// x x x x x
    /// x x x x x
    #[inline]
    pub fn fill_rows(&mut self, row: usize, n_rows: usize, value: T) {
        self.arr[row * COLS..(row + n_rows) * COLS].fill(value);
    }

    /// - - - - -
    /// - - x - -
    /// - - - - -
    #[inline]
    pub fn assign(&mut self, row: usize, col: usize, value: T) {
        self.arr[row * COLS + col] = value;
    }

    /// - - x - -
    /// - - x - -
    /// - - x - -
    #[inline]
    pub fn assign_rows(&mut self, row: usize, col: usize, n_rows: usize, value: T) {
        for n in 0..n_rows {
            self.arr[(row + n) * COLS + col] = value;
        }
    }

    #[inline]
    pub fn build(self) -> ndarray::Array2<T> {
        let shape = (self.rows(), COLS);
        ndarray::Array2::from_shape_vec(shape, self.arr).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn mutate() {
        let mut arr = Simple2DArray::<2, i32>::new(4);
        arr.fill(1, 3);
        assert_eq!(arr.build(), arr2(&[[0, 0], [3, 3], [0, 0], [0, 0]]));

        let mut arr = Simple2DArray::<2, i32>::new(4);
        arr.fill_rows(1, 2, 3);
        assert_eq!(arr.build(), arr2(&[[0, 0], [3, 3], [3, 3], [0, 0]]));

        let mut arr = Simple2DArray::<2, i32>::new(4);
        arr.assign(1, 1, 3);
        assert_eq!(arr.build(), arr2(&[[0, 0], [0, 3], [0, 0], [0, 0]]));

        let mut arr = Simple2DArray::<2, i32>::new(4);
        arr.assign_rows(1, 1, 2, 3);
        assert_eq!(arr.build(), arr2(&[[0, 0], [0, 3], [0, 3], [0, 0]]));
    }
}
