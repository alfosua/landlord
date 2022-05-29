use super::syntax_tree::{self, ReferencePath};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, multispace0, multispace1, satisfy, space0},
    combinator::{eof, map, opt, value},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

pub fn code_block<I, O, E, P>(parser: P) -> impl FnMut(I) -> nom::IResult<I, O, E>
where
    I: Clone
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputLength
        + nom::InputTakeAtPosition,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    P: nom::Parser<I, O, E>,
    E: nom::error::ParseError<I>,
{
    let opening_brace_prefix = tuple((multispace0, char('{'), multispace0));
    let closing_brace_suffix = tuple((multispace0, char('}'), multispace0));
    let target = delimited(opening_brace_prefix, parser, closing_brace_suffix);
    target
}

pub fn statement_termination<I>(input: I) -> nom::IResult<I, ()>
where
    I: Clone
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputLength
        + nom::InputTake
        + nom::InputTakeAtPosition,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
{
    value((), tuple((multispace0, char(';'), multispace0)))(input)
}

pub fn key_value_pairs<I, KO, VO, E, KP, VP>(
    key_parser: KP,
    value_parser: VP,
) -> impl FnMut(I) -> nom::IResult<I, std::collections::HashMap<KO, VO>, E>
where
    I: Clone
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    KO: Eq + std::hash::Hash,
    E: nom::error::ParseError<I>,
    KP: nom::Parser<I, KO, E>,
    VP: nom::Parser<I, VO, E>,
{
    let key_value_separator = tuple((multispace0, char('='), multispace0));
    let entry = separated_pair(key_parser, key_value_separator, value_parser);

    let entry_separator = alt((
        value((), tuple((space0, char(','), space0))),
        value((), many1(tuple((space0, line_ending, space0)))),
    ));

    let entry_list = separated_list0(entry_separator, entry);
    let hashmap = map(entry_list, |list: Vec<(KO, VO)>| {
        list.into_iter()
            .collect::<std::collections::HashMap<KO, VO>>()
    });

    hashmap
}

pub fn tagged_value<T, I, O, E, P>(tag_name: T, parser: P) -> impl FnMut(I) -> nom::IResult<I, O, E>
where
    T: Clone + nom::InputLength + nom::InputTake,
    I: Clone
        + nom::Compare<T>
        + nom::InputIter
        + nom::InputTakeAtPosition
        + nom::InputLength
        + nom::InputTake,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    E: nom::error::ParseError<I>,
    P: nom::Parser<I, O, E>,
{
    let tag = tag(tag_name);
    let prefix = pair(tag, multispace1);
    let prefixed_value = preceded(prefix, parser);
    prefixed_value
}

pub fn syntax_tree<I>(input: I) -> nom::IResult<I, syntax_tree::SyntaxTree>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    <I as nom::InputIter>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let eof = pair(multispace0, eof);
    let statements = delimited(multispace0, many0(statement), eof);
    let mut tree = map(statements, |statements| syntax_tree::SyntaxTree {
        statements,
    });
    tree(input)
}

pub fn statement<I>(input: I) -> nom::IResult<I, syntax_tree::Statement>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    <I as nom::InputIter>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let resource = map(resource_data, |data| syntax_tree::Statement::Resource(data));
    let mut statement_alt = alt((resource,));

    statement_alt(input)
}

pub fn resource_data<I>(input: I) -> nom::IResult<I, syntax_tree::ResourceData>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    <I as nom::InputIter>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let as_custom = resource_data_as_custom;
    let as_provider = resource_data_as_provider;
    let as_variable = resource_data_as_variable;
    let mut alt = alt((as_custom, as_provider, as_variable));
    alt(input)
}

pub fn resource_data_as_custom<I>(input: I) -> nom::IResult<I, syntax_tree::ResourceData>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    <I as nom::InputIter>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let resource_name = terminated(tagged_value("resource", name_identifier), multispace1);
    let resource_type_name = tagged_value("of", reference_path);
    let resource_modifiers = preceded(multispace1, resource_modifier_list);

    let resource_body_hashmap = key_value_pairs(name_identifier, expression);
    let resource_body = map(code_block(resource_body_hashmap), |data| Some(data));
    let resource_body_option = alt((value(None, statement_termination), resource_body));

    let resource_raw_data = tuple((
        resource_name,
        resource_type_name,
        resource_modifiers,
        resource_body_option,
    ));
    let mut resource_data = map(resource_raw_data, |(name, type_name, modifiers, body)| {
        syntax_tree::ResourceData::new(
            name,
            type_name,
            body,
            syntax_tree::ResourceClass::Custom,
            &modifiers,
        )
    });

    resource_data(input)
}

pub fn resource_data_as_provider<I>(input: I) -> nom::IResult<I, syntax_tree::ResourceData>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    <I as nom::InputIter>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let resource_name = terminated(tagged_value("provider", name_identifier), multispace1);
    let resource_type_name = tagged_value("of", reference_path);
    let resource_modifiers = preceded(multispace1, resource_modifier_list);

    let resource_body_hashmap = key_value_pairs(name_identifier, expression);
    let resource_body = map(code_block(resource_body_hashmap), |data| Some(data));
    let resource_body_option = alt((value(None, statement_termination), resource_body));

    let resource_raw_data = tuple((
        resource_name,
        resource_type_name,
        resource_modifiers,
        resource_body_option,
    ));
    let mut resource_data = map(resource_raw_data, |(name, type_name, modifiers, body)| {
        syntax_tree::ResourceData::new(
            name,
            type_name,
            body,
            syntax_tree::ResourceClass::Custom,
            &modifiers,
        )
    });

    resource_data(input)
}

pub fn resource_data_as_variable<I>(input: I) -> nom::IResult<I, syntax_tree::ResourceData>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::InputIter
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    <I as nom::InputIter>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let resource_name = terminated(tagged_value("variable", name_identifier), multispace1);

    let resource_body_hashmap = key_value_pairs(name_identifier, expression);
    let resource_body = map(code_block(resource_body_hashmap), |data| Some(data));
    let resource_body_option = alt((value(None, statement_termination), resource_body));

    let resource_raw_data = tuple((resource_name, resource_body_option));
    let mut resource_data = map(resource_raw_data, |(name, body)| {
        syntax_tree::ResourceData::new(
            name,
            ReferencePath {
                sequence: Vec::new(),
            },
            body,
            syntax_tree::ResourceClass::Variable,
            &Vec::new(),
        )
    });

    resource_data(input)
}

pub fn resource_modifier_list<I>(input: I) -> nom::IResult<I, Vec<syntax_tree::ResourceModifier>>
where
    I: Clone
        + nom::Compare<&'static str>
        + nom::InputTake
        + nom::InputIter
        + nom::InputTakeAtPosition
        + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
{
    let scoped_tag = tag("scoped");
    let scoped_item = value(syntax_tree::ResourceModifier::Scoped, scoped_tag);

    let resource_modifier_alt = alt((scoped_item,));
    let mut resource_modifier_list = separated_list0(multispace1, resource_modifier_alt);

    resource_modifier_list(input)
}

pub fn expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputTakeAtPosition
        + nom::InputTake
        + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    String: std::convert::From<I>,
{
    let mut expression_alt = alt((literal_expression, object_path_expression));
    expression_alt(input)
}

pub fn literal_expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: Clone
        + std::borrow::Borrow<str>
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputTakeAtPosition
        + nom::InputTake
        + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
    String: std::convert::From<I>,
{
    let boolean = boolean_literal_expression;
    let integer_number = integer_number_literal_expression;
    let float_number = float_number_literal_expression;
    let string = string_literal_expression;
    let mut expression_alt = alt((boolean, string, integer_number, float_number));
    expression_alt(input)
}

pub fn string_literal_expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: Clone + nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let mapper = |x| syntax_tree::Expression::Literal(syntax_tree::Literal::String(x));
    map(string_literal_data, mapper)(input)
}

pub fn string_literal_data<I>(input: I) -> nom::IResult<I, String>
where
    I: Clone + nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let characters = many0(satisfy(is_valid_singleline_string_char));
    let raw_value = delimited(quote, characters, quote);
    let mut value_map = map(raw_value, |chars| chars.iter().collect());
    value_map(input)
}

pub fn is_valid_singleline_string_char(c: char) -> bool {
    c != '"' && c != '\'' && c != '\n'
}

pub fn integer_number_literal_expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    let mapper = |x| {
        syntax_tree::Expression::Literal(syntax_tree::Literal::Number(
            syntax_tree::Number::Integer(x),
        ))
    };
    map(integer_number_literal_data, mapper)(input)
}

pub fn integer_number_literal_data<I>(input: I) -> nom::IResult<I, String>
where
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar,
    String: std::convert::From<I>,
{
    map(digit1, |s| String::from(s))(input)
}

pub fn float_number_literal_expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: std::borrow::Borrow<str>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputTakeAtPosition,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    let mapper = |x| {
        syntax_tree::Expression::Literal(syntax_tree::Literal::Number(
            syntax_tree::Number::FloatingPoint(x),
        ))
    };
    map(float_number_literal_data, mapper)(input)
}

pub fn float_number_literal_data<I>(input: I) -> nom::IResult<I, String>
where
    I: std::borrow::Borrow<str>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputTakeAtPosition,
    <I as nom::InputIter>::Item: nom::AsChar,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    let digits = separated_pair(digit1, char('.'), digit1);
    let mut data = map(digits, |(integer_part, fractional_part)| {
        [integer_part, fractional_part].join(".")
    });
    data(input)
}

pub fn boolean_literal_expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: Clone + nom::Compare<&'static str> + nom::InputTakeAtPosition + nom::InputTake,
    <I as nom::InputTakeAtPosition>::Item: Clone + nom::AsChar,
{
    let data = alt((value(true, tag("true")), value(false, tag("false"))));
    let mut expression = map(data, |data| {
        syntax_tree::Expression::Literal(syntax_tree::Literal::Boolean(data))
    });
    expression(input)
}

pub fn quote<I>(input: I) -> nom::IResult<I, char>
where
    I: Clone + nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    alt((char('"'), char('\'')))(input)
}

pub fn reference_path<'a, I>(input: I) -> nom::IResult<I, syntax_tree::ReferencePath>
where
    I: Clone
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputLength
        + nom::Compare<&'a str>
        + nom::InputTake,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let name_separator = pair(char(':'), char(':'));
    let name_sequence = separated_list1(name_separator, reference);
    let mut path_map = map(name_sequence, |sequence| syntax_tree::ReferencePath {
        sequence,
    });
    path_map(input)
}

pub fn name_identifier<I>(input: I) -> nom::IResult<I, syntax_tree::NameIdentifier>
where
    I: Clone + nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let first = satisfy(|c| c == '_' || c.is_alphabetic());
    let rest = many0(satisfy(|c| c == '_' || c.is_alphanumeric()));
    let pair = pair(first, rest);
    let mut data = map(pair, |(first, rest)| syntax_tree::NameIdentifier {
        value: [vec![first], rest].iter().flatten().collect(),
    });
    data(input)
}

pub fn reference<'a, I>(input: I) -> nom::IResult<I, syntax_tree::Reference>
where
    I: Clone
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputLength
        + nom::Compare<&'a str>
        + nom::InputTake,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    use syntax_tree::Reference::*;
    let from_name = reference_from_name;
    let from_super = value(Super, tag("super"));
    let from_land = value(Land, tag("land"));
    let mut reference_alt = alt((from_super, from_land, from_name));
    reference_alt(input)
}

pub fn reference_from_name<I>(input: I) -> nom::IResult<I, syntax_tree::Reference>
where
    I: Clone + nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter + nom::InputLength,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let mut reference = map(name_identifier, |name| syntax_tree::Reference::Name(name));
    reference(input)
}

pub fn object_path_expression<I>(input: I) -> nom::IResult<I, syntax_tree::Expression>
where
    I: Clone
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputLength
        + nom::InputTake,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let mapper = |op| syntax_tree::Expression::Object(op);
    map(object_path, mapper)(input)
}

pub fn object_path<I>(input: I) -> nom::IResult<I, syntax_tree::ObjectPath>
where
    I: Clone
        + nom::Compare<&'static str>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::InputIter
        + nom::InputLength
        + nom::InputTake,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    let name_separator = char('.');
    let name_sequence = separated_list0(&name_separator, name_identifier);
    let member_path = opt(preceded(&name_separator, name_sequence));
    let raw_data = pair(reference_path, member_path);
    let mut data = map(raw_data, |(object, member_path)| syntax_tree::ObjectPath {
        object,
        member_path,
    });
    data(input)
}
