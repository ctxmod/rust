//! Black-box tests of the public calculator API: rstest fixtures inject a
//! fresh instance into every test, and #[case] rows replace hand-written
//! tables. Each test gets its own state, so the suite runs the same under
//! cargo test (threads) and cargo nextest (one process per test).

use rstest::{fixture, rstest};
use test_fixtures::{Calculator, CalculatorError};

#[fixture]
fn calculator() -> Calculator {
    Calculator::new()
}

#[fixture]
fn calculator_with_history() -> Calculator {
    let mut calc = Calculator::new();
    calc.add(1, 2).unwrap();
    calc.multiply(3, 4).unwrap();
    calc
}

#[rstest]
fn fresh_calculator_has_no_history(calculator: Calculator) {
    assert!(calculator.history().is_empty());
}

#[rstest]
fn history_records_each_operation(calculator_with_history: Calculator) {
    let history = calculator_with_history.history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0], ("add".to_string(), 3));
    assert_eq!(history[1], ("multiply".to_string(), 12));
}

#[rstest]
#[case(2, 3, 5)]
#[case(0, 0, 0)]
#[case(-1, 1, 0)]
#[case(100, -50, 50)]
fn addition(mut calculator: Calculator, #[case] a: i64, #[case] b: i64, #[case] expected: i64) {
    assert_eq!(calculator.add(a, b).unwrap(), expected);
}

#[rstest]
#[case(i64::MAX, 1)]
#[case(i64::MIN, -1)]
fn addition_overflow_is_an_error(mut calculator: Calculator, #[case] a: i64, #[case] b: i64) {
    assert!(matches!(calculator.add(a, b), Err(CalculatorError::Overflow(_))));
}

#[rstest]
fn division_by_zero_is_an_error(mut calculator: Calculator) {
    assert!(matches!(calculator.divide(1, 0), Err(CalculatorError::DivisionByZero)));
}
