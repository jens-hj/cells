use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl std::fmt::Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Debug)]
pub struct Window<T: Clone + std::fmt::Debug> {
    pub grid: Grid<T>,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub enum GridError {
    EmptyGrid,
    UnequalRowLengths,
    OutOfBounds,
    SubgridBiggerThanGrid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T: Clone + std::fmt::Debug> {
    pub cells: Vec<Vec<T>>,
}

impl<T: Clone + std::fmt::Debug> Grid<T> {
    pub fn new(cells: Vec<Vec<T>>) -> Result<Self, GridError> {
        let grid = Grid { cells };
        grid.validate()?;
        Ok(grid)
    }

    pub fn validate(&self) -> Result<(), GridError> {
        if self.cells.is_empty() {
            return Err(GridError::EmptyGrid);
        }

        let expected_width = self.cells[0].len();
        if self.cells.iter().any(|row| row.len() != expected_width) {
            return Err(GridError::UnequalRowLengths);
        }

        Ok(())
    }

    pub fn dimensions(&self) -> Dimensions {
        let height = self.cells.len();
        let width = self.cells.first().map_or(0, |row| row.len());
        Dimensions { width, height }
    }

    pub fn get(&self, x: usize, y: usize) -> Result<&T, GridError> {
        self.cells
            .get(y)
            .ok_or(GridError::OutOfBounds)
            .and_then(|row| row.get(x).ok_or(GridError::OutOfBounds))
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Result<&mut T, GridError> {
        self.cells
            .get_mut(y)
            .ok_or(GridError::OutOfBounds)
            .and_then(|row| row.get_mut(x).ok_or(GridError::OutOfBounds))
    }

    pub fn get_subgrid(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Self, GridError> {
        let subgrid = self
            .cells
            .iter()
            .skip(y)
            .take(height)
            .map(|row| row.iter().skip(x).take(width).cloned().collect())
            .collect();
        Ok(Grid { cells: subgrid })
    }

    pub fn set_subgrid(&mut self, x: usize, y: usize, grid: Self) -> Result<(), GridError> {
        let Dimensions { width, height } = grid.dimensions();

        if self.dimensions().width < width || self.dimensions().height < height {
            return Err(GridError::SubgridBiggerThanGrid);
        }

        for (i, row) in self.cells.iter_mut().skip(y).take(height).enumerate() {
            for (j, cell) in row.iter_mut().skip(x).take(width).enumerate() {
                *cell = grid.cells[i][j].clone();
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.cells.iter().flat_map(|row| row.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.cells.iter_mut().flat_map(|row| row.iter_mut())
    }

    /// An windowed iterator that iterates over the grid in 2D windows of the given dimensions
    pub fn windowed(
        &'_ self,
        window_dimensions: Dimensions,
    ) -> impl Iterator<Item = Window<T>> + '_ {
        let Dimensions { width, height } = self.dimensions();
        (0..=height - window_dimensions.height).flat_map(move |y| {
            (0..=width - window_dimensions.width).map(move |x| Window {
                grid: self
                    .get_subgrid(x, y, window_dimensions.width, window_dimensions.height)
                    .unwrap(),
                x,
                y,
            })
        })
    }
}

/// Convert from vector of vectors to grid
impl<T: Clone + std::fmt::Debug> From<Vec<Vec<T>>> for Grid<T> {
    fn from(cells: Vec<Vec<T>>) -> Self {
        Grid::new(cells).unwrap()
    }
}

/// Display for grid as a matrix of strings
impl<T: Clone + std::fmt::Debug> std::fmt::Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.cells {
            write!(f, "{:?}\n", row)?;
        }
        Ok(())
    }
}
