use chumsky::prelude::*;

use crate::error::{SML_Result, SML_Error};
use crate::expression::Expression;
use crate::value::Value;
use crate::identifier::Identifier;
use crate::operation::UnaryOperation;
use crate::operation::BinaryOperation;


fn expr_parser() -> impl Parser<char, Expression, Error = Simple<char>> {
    let kw_nc = |s: &'static str| { text::keyword(s).map(|()| s.to_string() ) };

    let value = recursive(|val|{
        // TODO: floating point notation
        let num = text::int(10)
            .then(just('.').ignore_then(text::digits(10)).or_not())
            .map(| (sa, sb): (String, Option<String>) |{ 
                let s = match sb {
                    Some(sb) => sa + &sb,
                    None => sa,
                };
                Value::Number(s.parse().unwrap())
            })
            .padded()
            ;

        let bul = choice((
                kw_nc("true"),
                kw_nc("false"),
            ))
            .map(|s| Value::Bool(s == "true"))
            .padded()
            ;

        let str_ = just('"')
            .ignore_then(none_of("\"").repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .map(|s: String| Value::String(s))
            .padded()
            ;

        let list = just('[')
            .ignore_then(val.clone().then_ignore(just(',').or_not()).repeated().at_least(1))
            .then_ignore(just(']'))
            .collect::<Vec<Value>>()
            .map(|v| Value::List(v.into_iter().map(|vi| Box::new(vi)).collect() ))
            .padded()
            ;

        num.or(bul).or(str_).or(list)

    });

    recursive(|e| {

        let ident = choice((
                kw_nc("inputs"),
                kw_nc("outputs"),
                kw_nc("globals"),
            ))
            .then_ignore(just('.'))
            .then(text::ident())
            .map(|(sa, sb): (String, String)| { format!("{sa}.{sb}") })
            .map(|s: String| { Expression::Identifier(Identifier::from_str(s).unwrap())})
            .padded()
            ;

        let atom = value.map(|v| Expression::Value(v))
            .or(ident)
            .or(e.delimited_by(just('('), just(')')));

        let op = |c| just(c).padded();
        let op2 = |c| just(c).then(just('=')).padded();
        let opd = |c: char| just(c).then(just(c)).padded();
        // Why doesn't this work? :(
        // let opn = |s: &'static str| text::keyword(s).padded();

        let unary = op('-')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Expression::Unary(UnaryOperation::Negate, Box::new(rhs)));

        let curry_binary = |o: BinaryOperation | {
            |lhs: Expression, rhs: Expression| {
                Expression::Binary(o, Box::new(lhs), Box::new(rhs))
            }
        };

        let product = unary.clone()
            .then(choice((
                    op('*').to(curry_binary(BinaryOperation::Multiply)),
                    op('/').to(curry_binary(BinaryOperation::Divide)),
                    op('^').to(curry_binary(BinaryOperation::Power)),
                ))
                .then(unary)
                .repeated())
            .foldl(|lhs, (op, rhs)| op(lhs, rhs));

        let sum = product.clone()
            .then(op('+').to(curry_binary(BinaryOperation::Add))
                    .or(op('-').to(curry_binary(BinaryOperation::Subtract)))
                    .then(product)
                    .repeated())
            .foldl(|lhs, (op, rhs)| op(lhs, rhs));

        let misc_binary = sum.clone()
            .then(choice((
                    opd('&').to(curry_binary(BinaryOperation::And)),
                    opd('|').to(curry_binary(BinaryOperation::Or)),
                    op2('=').to(curry_binary(BinaryOperation::Equal)),
                    op2('!').to(curry_binary(BinaryOperation::NotEqual)),
                    op2('<').to(curry_binary(BinaryOperation::LessThanOrEqual)),
                    op2('>').to(curry_binary(BinaryOperation::GreaterThanOrEqual)),
                    op2('^').to(curry_binary(BinaryOperation::Contains)),
                    op('=').to(curry_binary(BinaryOperation::Assign)),
                    op('<').to(curry_binary(BinaryOperation::LessThan)),
                    op('>').to(curry_binary(BinaryOperation::GreaterThan)),
                ))
                .then(sum)
                .repeated())
            .foldl(|lhs, (op, rhs)| op(lhs, rhs));

        misc_binary.padded()
    }).then_ignore(end())
}


pub fn expr_from_str(s: &str, lineno: usize) -> SML_Result<Expression> {
    eprintln!("|{}|", s);
    let parser = expr_parser();
    match parser.parse(s) {
        Err(e) => Err(SML_Error::CompilerError(format!("Failed to parse expr on line {lineno}: {e:?}"))),
        Ok(e) => Ok(e),
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_expr_parse_1() {
        let i = "1";
        let o = expr_from_str(i, 0).unwrap();
        assert!(matches!(o, Expression::Value(Value::Number(_))));
    }

    #[test]
    fn test_expr_parse_2() {
        let i = "  true ";
        let o = expr_from_str(i, 0).unwrap();
        assert!(matches!(o, Expression::Value(Value::Bool(_))));
    }

    #[test]
    fn test_expr_parse_3() {
        let i = "1 + 1";
        let o = expr_from_str(i, 0).unwrap();
        assert!(matches!(o, Expression::Binary(BinaryOperation::Add, _, _)));
    }

    #[test]
    fn test_expr_parse_4() {
        let i = "1 == 1";
        let o = expr_from_str(i, 0).unwrap();
        assert!(matches!(o, Expression::Binary(BinaryOperation::Equal, _, _)));
    }

    #[test]
    fn test_expr_parse_5() {
        let i = "1 ^= 1";
        let o = expr_from_str(i, 0).unwrap();
        assert!(matches!(o, Expression::Binary(BinaryOperation::Contains, _, _)));
    }

    #[test]
    fn test_expr_parse_identifier_contains_int() {
        let i = "inputs.foo ^= 1";
        let o = expr_from_str(i, 0).unwrap();
        match o {
            Expression::Binary(BinaryOperation::Contains, l, r) => {
                match (*l, *r) {
                    (Expression::Identifier(_), Expression::Value(Value::Number(_))) => (),
                    _ => {panic!()} ,
                }
            },
            _ => {panic!()}
        }
    }

    #[test]
    fn test_expr_parse_string() {
        let i = "inputs.foo = \"ooh thing another bar\"";
        let o = expr_from_str(i, 0).unwrap();
        match o {
            Expression::Binary(BinaryOperation::Assign, l, r) => {
                match (*l, *r) {
                    (Expression::Identifier(_), Expression::Value(Value::String(_))) => (),
                    _ => {panic!()},
                }
            },
            _ => {panic!()}
        }
    }

    #[test]
    fn test_expr_parse_list() {
        let i = "inputs.foo = [1, 2, 3]";
        let o = expr_from_str(i, 0).unwrap();
        match o {
            Expression::Binary(BinaryOperation::Assign, l, r) => {
                match (*l, *r) {
                    (Expression::Identifier(_), Expression::Value(Value::List(_))) => (),
                    _ => {panic!()},
                }
            },
            _ => {panic!()}
        }
    }

}
