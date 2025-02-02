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
pub enum GridError {
    EmptyGrid,
    UnequalRowLengths,
    OutOfBounds,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T> {
    pub cells: Vec<Vec<T>>,
}

impl<T> Grid<T> {
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

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.cells.iter().flat_map(|row| row.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.cells.iter_mut().flat_map(|row| row.iter_mut())
    }
}
