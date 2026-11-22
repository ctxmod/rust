//! Calculator module for testing demonstrations.

use thiserror::Error;

/// Calculator errors.
#[derive(Debug, Error)]
pub enum CalculatorError {
    #[error("division by zero")]
    DivisionByZero,

    #[error("overflow: {0}")]
    Overflow(String),
}

/// Simple calculator for arithmetic operations.
#[derive(Debug, Clone)]
pub struct Calculator {
    history: Vec<(String, i64)>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Add two numbers.
    ///
    /// ```rust
    /// # use test_fixtures::{Calculator, CalculatorError};
    /// let mut calc = Calculator::new();
    /// assert_eq!(calc.add(2, 3).unwrap(), 5);
    /// assert!(matches!(calc.add(i64::MAX, 1), Err(CalculatorError::Overflow(_))));
    /// ```
    pub fn add(&mut self, a: i64, b: i64) -> Result<i64, CalculatorError> {
        let result = a.checked_add(b).ok_or_else(|| {
            CalculatorError::Overflow(format!("{} + {}", a, b))
        })?;
        self.history.push(("add".to_string(), result));
        Ok(result)
    }

    /// Subtract two numbers.
    pub fn subtract(&mut self, a: i64, b: i64) -> Result<i64, CalculatorError> {
        let result = a.checked_sub(b).ok_or_else(|| {
            CalculatorError::Overflow(format!("{} - {}", a, b))
        })?;
        self.history.push(("subtract".to_string(), result));
        Ok(result)
    }

    /// Multiply two numbers.
    pub fn multiply(&mut self, a: i64, b: i64) -> Result<i64, CalculatorError> {
        let result = a.checked_mul(b).ok_or_else(|| {
            CalculatorError::Overflow(format!("{} * {}", a, b))
        })?;
        self.history.push(("multiply".to_string(), result));
        Ok(result)
    }

    /// Divide two numbers.
    pub fn divide(&mut self, a: i64, b: i64) -> Result<i64, CalculatorError> {
        if b == 0 {
            return Err(CalculatorError::DivisionByZero);
        }
        let result = a / b;
        self.history.push(("divide".to_string(), result));
        Ok(result)
    }

    /// Get operation history.
    pub fn history(&self) -> &[(String, i64)] {
        &self.history
    }

    /// Clear history.
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}
