#[cfg(test)]
mod tests {
    use crate::{roll::*, Result};

    #[test]
    fn roll_display() {
        let roll1 = Roll {
            result: 1,
            keep: false,
        };
        let roll2 = Roll {
            result: 1,
            keep: true,
        };

        assert_eq!(roll1.to_string(), "1");
        assert_eq!(roll2.to_string(), "**1**");
        println!("{}", roll1);
        println!("{}", roll2);
    }

    #[test]
    fn roll_output_display() {
        let output = Output {
            rolls: vec![
                Roll {
                    result: 1,
                    keep: false,
                },
                Roll {
                    result: 2,
                    keep: true,
                },
            ],
            total: 2,
        };

        assert_eq!(output.to_string(), "2 [1, **2**]");
        println!("{}", output);
    }

    #[test]
    fn roll_cmp() {
        let mut rolls = vec![
            Roll {
                result: 2,
                keep: false,
            },
            Roll {
                result: 1,
                keep: false,
            },
            Roll {
                result: 1,
                keep: true,
            },
            Roll {
                result: 2,
                keep: true,
            },
        ];

        rolls.sort();

        assert_eq!(
            vec![1, 1, 2, 2],
            rolls.iter().map(|r| r.result).collect::<Vec<_>>()
        );
    }

    /// Macro for testing the parser.
    /// The macro will create a test function with the name of the first argument,
    /// and the second argument will be the expression to parse.
    /// The third argument is a closure that will be called with the output of the parse.
    ///
    /// # Examples
    ///
    /// ```
    /// parser_test! {basic, "1d20", (|output: Rolloutput| {
    ///     assert!((1..=20).contains(&output.total));
    /// })}
    /// ```
    ///
    /// output in
    ///
    /// ```
    /// #[test]
    /// fn basic() -> Result<()> {
    ///    let expr = "1d20";
    ///    println!("Testing parse of `{}`", expr);
    ///
    ///    let output = eval(expr)?;
    ///
    ///    assert!((1..=20).contains(&output.total));
    ///    Ok(())
    /// }
    /// ```
    macro_rules! parser_test {
        ($name:ident, $expr:expr, $closure:expr) => {
            #[test]
            fn $name() -> Result<()> {
                let expr: &str = $expr;
                println!("Testing parse of `{}`", expr);

                let ast = parser::expression(expr.trim()).map_err(|e| {
                    #[cfg(feature = "trace")]
                    println!("Failed to parse `{}`: {:#?}", expr, e);
                    e
                })?;

                #[cfg(feature = "trace")]
                println!("AST: {:#?}", ast);

                let output = ast.eval()?;

                let closure: fn(Output) = $closure;

                closure(output);
                Ok(())
            }
        };
    }

    parser_test! {basic, "1d20", |output| {
        assert!((1..=20).contains(&output.total));
    }}

    parser_test! {addition, "1 + 1", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(2, output.total);
    }}

    parser_test! {subtraction, "1 - 1", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(0, output.total);
    }}

    parser_test! {multiplication, "2 * 3", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(6, output.total);
    }}

    parser_test! {division, "6 / 3", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(2, output.total);
    }}

    parser_test! {negative, "-6", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(-6, output.total);
    }}

    parser_test! {missing_count, "d4", |output| {
        assert_eq!(1, output.rolls.len());
        assert!((1..=4).contains(&output.total));
    }}

    parser_test! {keep, "3d20k2", |output| {
        assert_eq!(3, output.rolls.len());
        assert!((2..=40).contains(&output.total));
    }}

    parser_test! {drop, "3d20d2", |output| {
        assert_eq!(3, output.rolls.len());
        println!("rolls: {:#?}", output.rolls);
        assert!((1..=20).contains(&output.total));
    }}

    // TODO: Fix these tests
    // parser_test! {keep_and_drop, "3d20k2d1", |output| {
    //     assert_eq!(1, output.rolls.len());
    //     assert!((2..=40).contains(&output.total));
    // }}

    // parser_test! {keep_and_drop2, "3d20d1k2", |output| {
    //     assert_eq!(1, output.rolls.len());
    //     assert!((2..=40).contains(&output.total));
    // }}

    parser_test! {arithmetic1, "1 + 3 * 5", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(16, output.total);
    }}

    parser_test! {arithmetic2, "1 + 3 * 5 - 2", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(14, output.total);
    }}

    parser_test! {arithmetic3, "1 + 3 * 5 - 2 / 2 - 1", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(14, output.total);
    }}

    parser_test! {arithmetic_with_parens, "(1 + 3) * 5 - 2 / ( 2 - 1)", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(18, output.total);
    }}

    parser_test! {arithmetic_with_dice, "1d4 + 2", |output| {
        assert_eq!(1, output.rolls.len());
        assert!((3..=6).contains(&output.total));
    }}

    parser_test! {arithmetic_with_dice2, "1 + 2d4", |output| {
        assert_eq!(2, output.rolls.len());
        assert!((3..=9).contains(&output.total));
    }}

    parser_test! {arithmetic_with_dice3, "1d4 + 2d4", |output| {
        assert_eq!(3, output.rolls.len());
        assert!((3..=12).contains(&output.total));
    }}

    parser_test! {arithmetic_with_dice4, "1d4 + 2d4 * 3d4", |output| {
        assert_eq!(6, output.rolls.len());
        assert!((7..=100).contains(&output.total));
    }}

    parser_test! {parens, "1d(4 + 2)", |output| {
        assert_eq!(1, output.rolls.len());
        assert!((1..=6).contains(&output.total));
    }}

    parser_test! {parens2, "1d(4 + 2) * 3", |output| {
        assert_eq!(1, output.rolls.len());
        assert!((3..=18).contains(&output.total));
    }}

    parser_test! {right_parens, "1 + (2d4)", |output| {
        assert_eq!(2, output.rolls.len());
        assert!((3..=9).contains(&output.total));
    }}

    parser_test! {number, "1", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(1, output.total);
    }}

    parser_test! {negative_number, "-1", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(-1, output.total);
    }}

    parser_test! {negative_number2, "2 + -1", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(1, output.total);
    }}

    parser_test! {negative_parens, "-(1+3)", |output| {
        assert_eq!(0, output.rolls.len());
        assert_eq!(-4, output.total);
    }}
}
