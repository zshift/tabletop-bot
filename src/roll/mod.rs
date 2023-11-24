mod tests;

use peg::error::ParseError;
use rand::Rng;
use std::{error::Error, fmt::Display};

type Result<T> = std::result::Result<T, RollError>;

#[derive(Clone, Debug)]
pub enum RollError {
    InvalidExpression,
    InvalidCount,
    InvalidSides,
    InvalidKeep,
    InvalidDrop,
    DivideByZero,
    ParseError(String),
}

impl From<ParseError<peg::str::LineCol>> for RollError {
    fn from(e: ParseError<peg::str::LineCol>) -> Self {
        RollError::ParseError(e.to_string())
    }
}

impl Display for RollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cause = match self {
            RollError::InvalidExpression => "Invalid expression",
            RollError::InvalidCount => "Count must be at least 1",
            RollError::InvalidSides => "Sides must be at least 2",
            RollError::InvalidKeep => "Keep must be at least 1",
            RollError::InvalidDrop => "Drop must be at least 1",
            RollError::DivideByZero => "Cannot divide by zero",
            RollError::ParseError(cause) => cause.as_str(),
        };

        write!(f, "Roll failed. Cause: {:#?}", cause)
    }
}

impl Error for RollError {}

/// Evaluates the expression, and rolls dice in compliance with that expression.
///
/// # Syntax
/// The syntax is based on the [dice notation](https://en.wikipedia.org/wiki/Dice_notation) used in
/// tabletop games.
///
/// # Examples
///
/// **Basic roll**
/// ```
/// use dnd_bot::roll::RollResults;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let results: RollResults = roll::eval("1d20")?;
///
/// assert_eq!(results.rolls.len(), 1);
/// assert!((1..=20).contains(&results.total));
/// # Ok(())
/// # }
/// ```
///
/// **Arithmetic on roll results**
/// ```
/// # use dnd_bot::roll::RollResults;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let results: RollResults = roll::eval("3d4 * 5")?;
///
/// assert_eq!(results.rolls.len(), 3);
/// assert!((15..=60).contains(&results.total));
/// # Ok(())
/// # }
/// ```
///
/// **Keep highest**
/// ```
/// # use dnd_bot::roll::RollResults;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let results: RollResults = roll::eval("3d4k2")?;
///
/// assert_eq!(results.rolls.len(), 3);
/// assert!((2..=8).contains(&results.total));
/// # Ok(())
/// # }
/// ```
///
pub fn eval(expression: &str) -> Result<Output> {
    let ast = parser::expression(expression.trim())?;
    ast.eval()
}

trait Traceable<T> {
    fn trace(self) -> T;
}

impl<T> Traceable<T> for T
where
    T: std::fmt::Debug,
{
    #[inline]
    fn trace(self) -> T {
        #[cfg(feature = "trace")]
        println!("{:#?}", self);
        self
    }
}

peg::parser! {
    /// # Roll Parser
    ///
    /// Parses and evaluates a dice roll expression according to the following grammar.
    ///
    /// ## Backus–Naur form
    ///
    /// ```bnf
    /// <Expression>     ::= <Term>? <_> <Sum>?
    /// <Sum>            ::= <AddOp> <_> <Term> <Sum>?
    ///
    /// <Term>           ::= <Factor> <_> <Product>?
    /// <Product>        ::= <MulOp> <_> <Factor> <Product>?
    ///
    /// <Factor>         ::= <Integer> | <DiceRoll> | <NestedExpr>
    ///
    /// <DiceRoll>       ::= <RollExpression>? "d" <RollExpression> <Keep>? <Drop>?
    /// <RollExpression> ::= <Number> | <NestedExpr>
    ///
    /// <NestedExpr>     ::= "(" <_> <Expression> <_> ")"
    ///
    /// <AddOp>          ::= "+" | "-"
    /// <MulOp>          ::= "*" | "/" | "%"
    ///
    /// <KeepLow>        ::= "kl" <RollExpression>
    /// <KeepHigh>       ::= ("k" | "kh") <RollExpression>
    /// <Keep>           ::= <KeepHigh> | <KeepLow>
    ///
    /// <DropLow>        ::= ("d" | "dl") <RollExpression>
    /// <DropHigh>       ::= "dh" <RollExpression>
    /// <Drop>           ::= <DropHigh> | <DropLow>
    ///
    /// <Integer>        ::= "-"? <Number>
    /// <Number>         ::= <Digit> <Number>?
    /// <Digit>          ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
    ///
    /// # Whitespace
    /// <_>              ::= (" " | "\t")*
    /// ```
    pub grammar parser() for str {

        // To ignore whitespace
        rule _ = [' ' | '\t' ]*

        // <Expression> ::= <Product> <Sum'>?
        pub rule expression() -> Expression = t:term()? _ s:sum()? { Expression::new(t, s).trace() }

        // <Sub'> ::= <AddOp> <_> <Product> <Sub'>?
        rule sum() -> Sum = op:add_op() _ p:term() s:sum()? { Sum::new(op, p, s).trace() }

        // <Term> ::= <Factor> <Product>?
        rule term() -> Term = f:factor() _ p:product()? { Term::new(f, p).trace() }

        // <Product> ::= MulOp <_> <Factor> <Product>?
        rule product() -> Product = op:mul_op() _ f:factor() p:product()? { Product::new(op, f, p).trace()}

        // <Factor> ::= <DiceRoll> | <Integer> | <NestedExpr>
        rule factor() -> Factor
            = dr:dice_roll() { Factor::DiceRoll(Box::new(dr)).trace() }
            / i:integer() { Factor::Integer(i).trace()}
            / ne:nested_expression() { Factor::Expression(Box::new(ne)).trace() }

        // <Integer> ::= "-"? <Number>
        rule integer() -> i32
            = neg:"-"? n:number() {
                let n = n as i32;
                if neg.is_some() { -n } else { n }
            }

        // <Number> ::= <Digit> <Number>?
        // <Digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
        rule number() -> u32 = n:$(['0'..='9']+) { n.parse::<u32>().unwrap().trace() }

        // <NestedExpr> ::= "(" <_> <Expression> <_> ")"
        rule nested_expression() -> Expression = "(" _ e:expression() _ ")" { e.trace() }

        /// Rolls the dice :D
        ///
        /// ```bnf
        /// <DiceRoll> ::= <RollExpression>? "d" <RollExpression> <Keep>? <Drop>?
        /// ```
        rule dice_roll() -> DiceRoll
            = count:roll_expression()? "d" sides:roll_expression() keep:keep()? drop:drop()? {
                DiceRoll::new(count, sides, keep, drop).trace()
            }

        // <RollExpression> ::= <Number> | "(" <_> <Expression> <_> ")"
        rule roll_expression() -> RollExpr
            = ne:nested_expression() { RollExpr::Expression(ne).trace() }
            / n:number() { RollExpr::Number(n).trace() }


        // <KeepLow> ::= "kl" <RollExpression>
        // <KeepHigh> ::= ("k" | "kh") <RollExpression>
        // <Keep> ::= <KeepHigh> | <KeepLow>
        rule keep() -> Keep
            = "kl" e:roll_expression() { Keep::Low(Box::new(e)).trace() }
            / ("k" / "kh") e:roll_expression() { Keep::High(Box::new(e)).trace() }

        // <DropLow> ::= ("d" | "dl") <RollExpression>
        // <DropHigh> ::= "dh" <RollExpression>
        // <Drop> ::= <DropHigh> | <DropLow>
        rule drop() -> Drop
            = "dh" e:roll_expression() { Drop::High(Box::new(e)).trace() }
            / ("d" / "dl") e:roll_expression() { Drop::Low(Box::new(e)).trace() }

        // <AddOp> ::= "+" | "-"
        rule add_op() -> AddOp
            = "+" { AddOp::Add }
            / "-" { AddOp::Sub }

        // <MulOp> ::= "*" | "/" | "%"
        rule mul_op() -> MulOp
            = "*" { MulOp::Mul }
            / "/" { MulOp::Div }
            / "%" { MulOp::Mod }
    }
}

// TODO: Not sure I need this, but it's convenient for now.
trait Eval {
    fn eval(self) -> Result<Output>;
}

/// The result of a roll, and whether or not it is kept.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct Roll {
    /// The result of the roll.
    pub result: u32,
    /// Whether or not the roll is kept.
    pub keep: bool,
}

impl Ord for Roll {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.result.cmp(&other.result)
    }
}

impl Display for Roll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.keep {
            write!(f, "**{}**", self.result)
        } else {
            write!(f, "{}", self.result)
        }
    }
}

// region: RollResults

/// The output of evaluating a roll expression.
#[derive(Clone, Debug)]
pub struct Output {
    /// The individual rolls that were made.
    pub rolls: Vec<Roll>,
    /// The total of evaluated expression.
    pub total: i32,
}

impl Output {
    pub fn of_num(num: i32) -> Self {
        Self {
            rolls: Vec::new(),
            total: num,
        }
    }

    pub fn check_greater_than(self, test: i32) -> Result<Output> {
        if self.total > test {
            Ok(self)
        } else {
            Err(RollError::InvalidExpression)
        }
    }

    #[inline(always)]
    fn infix<T>(left: Output, right: Output, op: T) -> Output
    where
        T: FnOnce(i32, i32) -> i32,
    {
        Output {
            rolls: vec![left.rolls, right.rolls].concat(),
            total: op(left.total, right.total),
        }
    }
}

impl std::ops::Add for Output {
    type Output = Output;

    fn add(self, rhs: Self) -> Self::Output {
        Output::infix(self, rhs, std::ops::Add::add)
    }
}

impl std::ops::Sub for Output {
    type Output = Output;

    fn sub(self, rhs: Self) -> Self::Output {
        Output::infix(self, rhs, std::ops::Sub::sub)
    }
}

impl std::ops::Mul for Output {
    type Output = Output;

    fn mul(self, rhs: Self) -> Self::Output {
        Output::infix(self, rhs, std::ops::Mul::mul)
    }
}

impl std::ops::Div for Output {
    type Output = Result<Output>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.total == 0 {
            return Err(RollError::DivideByZero);
        }

        Ok(Output::infix(self, rhs, std::ops::Div::div))
    }
}

impl std::ops::Rem for Output {
    type Output = Result<Output>;

    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.total == 0 {
            return Err(RollError::DivideByZero);
        }

        Ok(Output::infix(self, rhs, std::ops::Rem::rem))
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rolls.is_empty() {
            return write!(f, "{}", self.total);
        } else {
            write!(
                f,
                "{} [{}]",
                self.total,
                self.rolls
                    .iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

// endregion: RollResults

#[derive(Clone, Debug)]
pub enum RollExpr {
    Number(u32),
    Expression(Expression),
}

impl Eval for RollExpr {
    fn eval(self) -> Result<Output> {
        match self {
            RollExpr::Number(n) => Ok(Output::of_num(n as i32)),
            RollExpr::Expression(e) => e.eval(),
        }
    }
}

// region: Sum

#[derive(Clone, Debug)]
pub struct Expression {
    pub term: Box<Term>,
    pub sum: Option<Box<Sum>>,
}

impl Expression {
    pub fn new(term: Option<Term>, sum: Option<Sum>) -> Self {
        if let Some(term) = term {
            Self {
                term: Box::new(term),
                sum: sum.map(Box::new),
            }
        } else {
            Self {
                term: Box::new(Term::new(Factor::Integer(0), None)),
                sum: sum.map(Box::new),
            }
        }
    }
}

impl Default for Expression {
    fn default() -> Self {
        Self {
            term: Box::new(Term::new(Factor::Integer(0), None)),
            sum: None,
        }
    }
}

impl Eval for Expression {
    fn eval(self) -> Result<Output> {
        let product = self.term.eval()?;

        if let Some(sum) = self.sum {
            sum.eval(product)
        } else {
            Ok(product)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sum {
    op: AddOp,
    right: Box<Term>,
    extra: Option<Box<Sum>>,
}

impl Sum {
    pub fn new(op: AddOp, right: Term, extra: Option<Sum>) -> Self {
        Self {
            op,
            right: Box::new(right),
            extra: extra.map(Box::new),
        }
    }

    pub fn eval(self, left: Output) -> Result<Output> {
        let right = self.right.eval()?;
        let sum = match self.op {
            AddOp::Add => left + right,
            AddOp::Sub => left - right,
        };

        if let Some(extra) = self.extra {
            extra.eval(sum)
        } else {
            Ok(sum)
        }
    }
}

impl Default for Sum {
    fn default() -> Self {
        // TODO: this is a hack to get around the fact that the parser doesn't support unary
        Self {
            op: AddOp::Add,
            right: Box::new(Term::new(Factor::Integer(0), None)),
            extra: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AddOp {
    Add,
    Sub,
}

// endregion: Sum

// region: Product

#[derive(Clone, Debug)]
pub struct Term {
    pub factor: Box<Factor>,
    pub product: Option<Box<Product>>,
}

impl Term {
    pub fn new(factor: Factor, product: Option<Product>) -> Self {
        Self {
            factor: Box::new(factor),
            product: product.map(Box::new),
        }
    }
}

impl Eval for Term {
    fn eval(self) -> Result<Output> {
        let left = self.factor.eval()?;

        if let Some(product) = self.product {
            product.eval(left)
        } else {
            Ok(left)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Product {
    op: MulOp,
    right: Factor,
    extra: Option<Box<Product>>,
}

impl Product {
    pub fn new(op: MulOp, right: Factor, extra: Option<Product>) -> Self {
        Self {
            op,
            right,
            extra: extra.map(Box::new),
        }
    }

    pub fn eval(self, left: Output) -> Result<Output> {
        let right = self.right.eval()?;

        let product = match self.op {
            MulOp::Mul => left * right,
            MulOp::Div => (left / right)?,
            MulOp::Mod => (left % right)?,
        };

        if let Some(extra) = self.extra {
            extra.eval(product)
        } else {
            Ok(product)
        }
    }
}

impl Default for Product {
    fn default() -> Self {
        // TODO: this is a hack to get around the fact that the parser doesn't support unary
        Self {
            op: MulOp::Mul,
            right: Factor::Integer(1),
            extra: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

// endregion: Product

#[derive(Clone, Debug)]
pub enum Factor {
    Integer(i32),
    Expression(Box<Expression>),
    DiceRoll(Box<DiceRoll>),
}

impl Eval for Factor {
    fn eval(self) -> Result<Output> {
        match self {
            Factor::Integer(n) => Ok(Output::of_num(n)),
            Factor::Expression(expr) => expr.eval(),
            Factor::DiceRoll(dice_roll) => dice_roll.eval(),
        }
    }
}

// region:

/// Rolls a number of dice with the given number of sides.
pub fn roll_dice(count: u32, sides: u32) -> Vec<Roll> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| Roll {
            result: rng.gen_range(1..=sides),
            keep: true,
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct DiceRoll {
    pub count: Option<Box<RollExpr>>,
    pub sides: Box<RollExpr>,
    pub keep: Option<Keep>,
    pub drop: Option<Drop>,
}

impl DiceRoll {
    pub fn new(
        count: Option<RollExpr>,
        sides: RollExpr,
        keep: Option<Keep>,
        drop: Option<Drop>,
    ) -> Self {
        Self {
            count: count.map(Box::new),
            sides: Box::new(sides),
            keep,
            drop,
        }
    }

    fn high_to_low(rolls: &mut [&mut Roll]) {
        rolls.sort_by(|a, b| b.cmp(a))
    }

    fn low_to_high(rolls: &mut [&mut Roll]) {
        rolls.sort()
    }

    fn total(rolls: &[Roll]) -> i32 {
        rolls
            .iter()
            .filter(|r| r.keep)
            .map(|r| r.result as i32)
            .sum()
    }
}

impl Eval for DiceRoll {
    fn eval(self) -> Result<Output> {
        let count = if let Some(count) = self.count {
            count
                .eval()?
                .check_greater_than(0)
                .map_err(|_| RollError::InvalidCount)?
                .total as u32
        } else {
            1
        };

        let sides = self
            .sides
            .eval()?
            .check_greater_than(1)
            .map_err(|_| RollError::InvalidSides)?
            .total as u32;

        let mut rolls = roll_dice(count, sides);

        let keep_rolls = if let Some(keep) = self.keep {
            let sort = match keep {
                Keep::High(_) => Self::high_to_low,
                Keep::Low(_) => Self::low_to_high,
            };

            let results = keep
                .eval()?
                .check_greater_than(0)
                .map_err(|_| RollError::InvalidKeep)?;

            let num_to_keep = results.total as usize;
            let mut to_keep: Vec<&mut Roll> = rolls.iter_mut().collect();

            // reverse sort by result
            sort(&mut to_keep);
            to_keep
                .iter_mut()
                .skip(num_to_keep)
                .for_each(|k| k.keep = false);

            results.rolls.clone()
        } else {
            Vec::new()
        };

        let drop_rolls = if let Some(drop) = self.drop {
            let sort = match drop {
                Drop::High(_) => Self::high_to_low,
                Drop::Low(_) => Self::low_to_high,
            };

            let results = drop
                .eval()?
                .check_greater_than(0)
                .map_err(|_| RollError::InvalidDrop)?;

            let num_to_drop = results.total as usize;
            let mut to_drop: Vec<&mut Roll> = rolls.iter_mut().collect();

            // reverse sort by result
            sort(&mut to_drop);
            to_drop
                .iter_mut()
                .take(num_to_drop)
                .for_each(|drop| drop.keep = false);

            results.rolls.clone()
        } else {
            Vec::new()
        };

        let total = Self::total(&rolls) as i32;

        Ok(Output {
            rolls: vec![rolls, keep_rolls, drop_rolls].concat(),
            total,
        })
    }
}

// endregion: DiceRoll

// region: Keep

#[derive(Clone, Debug)]
pub enum Keep {
    High(Box<RollExpr>),
    Low(Box<RollExpr>),
}

impl Eval for Keep {
    fn eval(self) -> Result<Output> {
        match self {
            Keep::High(results) => results.eval(),
            Keep::Low(results) => results.eval(),
        }
    }
}

// endregion: Keep

// region: Drop

#[derive(Clone, Debug)]
pub enum Drop {
    High(Box<RollExpr>),
    Low(Box<RollExpr>),
}

impl Eval for Drop {
    fn eval(self) -> Result<Output> {
        match self {
            Drop::High(results) => results.eval(),
            Drop::Low(results) => results.eval(),
        }
    }
}

// endregion: Drop
