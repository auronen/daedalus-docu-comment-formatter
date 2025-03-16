use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{
        alpha1, alphanumeric1, char, line_ending, multispace0, multispace1, newline,
        not_line_ending, space0,
    },
    combinator::{opt, recognize},
    error::Error,
    multi::{many0, many0_count, separated_list0},
    sequence::{delimited, pair, terminated},
    Finish, IResult,
};

#[derive(Debug)]
struct DocuComment<'a> {
    description: Option<&'a str>,
    infos: Vec<DocuInfo<'a>>,
    func_string: &'a str,
    func_name: &'a str,
}

#[derive(Debug)]
enum DocuInfo<'a> {
    Parameter(&'a str, &'a str),
    Global(&'a str, &'a str),
    Return(&'a str),
}

fn parse_infos(input: &str) -> IResult<&str, Vec<DocuInfo>> {
    many0(terminated(parse_docu_info, newline))(input)
}

fn parse_docu_info(input: &str) -> IResult<&str, DocuInfo> {
    delimited(
        opt(parse_empty_line),
        alt((parse_param, parse_global, parse_return)),
        opt(parse_empty_line),
    )(input)
}

fn parse_param(input: &str) -> IResult<&str, DocuInfo> {
    let (input, _) = tag("///")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("@param")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, param_name) = take_until(" ")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, description) = not_line_ending(input)?;
    Ok((input, DocuInfo::Parameter(param_name, description)))
}

fn parse_global(input: &str) -> IResult<&str, DocuInfo> {
    let (input, _) = tag("///")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("@global")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, global_name) = take_until(" ")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, description) = not_line_ending(input)?;
    Ok((input, DocuInfo::Global(global_name, description)))
}

fn parse_return(input: &str) -> IResult<&str, DocuInfo> {
    let (input, _) = tag("///")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("@return")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, description) = not_line_ending(input)?;
    Ok((input, DocuInfo::Return(description)))
}

impl<'a> DocuComment<'a> {
    pub fn generate_md(&self) -> String {
        let mut md = String::with_capacity(150);
        md.push_str(&format!("### `{}`\n", self.func_name));
        md.push_str(&format!("!!! function \"`{}`\"\n", self.func_name));
        if let Some(desc) = &self.description {
            md.push_str(&format!("    {}\n", desc));
        }
        md.push_str(&format!("    ```dae\n    {}\n    ```\n", self.func_string));
        if self.has_params() {
            md.push_str("\n    **Parameters**  \n\n");
            self.infos.iter().for_each(|info| {
                if let DocuInfo::Parameter(name, desc) = info {
                    md.push_str(&format!("    - `#!dae {}` - {}\n", name, desc))
                }
            });
        }
        if self.has_globals() {
            md.push_str("\n    **Globals**  \n\n");
            self.infos.iter().for_each(|info| {
                if let DocuInfo::Global(name, desc) = info {
                    md.push_str(&format!("    - `#!dae {}` - {}\n", name, desc))
                }
            });
        }

        if self.has_return() {
            md.push_str("\n    **Return value**  \n");
            for info in &self.infos {
                if let DocuInfo::Return(desc) = info {
                    md.push_str(&format!("    The function returns {}\n", desc));
                    break;
                }
            }
        }
        md
    }

    fn has_params(&self) -> bool {
        self.infos
            .iter()
            .any(|info| matches!(info, DocuInfo::Parameter(..)))
    }

    fn has_globals(&self) -> bool {
        self.infos
            .iter()
            .any(|info| matches!(info, DocuInfo::Global(..)))
    }

    fn has_return(&self) -> bool {
        self.infos
            .iter()
            .any(|info| matches!(info, DocuInfo::Return(..)))
    }
}

fn parse_description(input: &str) -> IResult<&str, Option<&str>> {
    let (input, desc) = alt((take_until("///\n"), take_until("func")))(input)?;
    let desc = desc
        .strip_prefix("///")
        .expect("this string to start with ///")
        .trim();
    if desc.is_empty() {
        Ok((input, None))
    } else {
        Ok((input, Some(desc)))
    }
}

fn parse_empty_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("///")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_func(input: &str) -> IResult<&str, &str> {
    let (input, func) = take_until("{};")(input)?;
    let (input, _) = tag("{};")(input)?;
    // Ok((input, format!("{} {{}};", func)))
    Ok((input, func))
}


// super lazy here
fn parse_function_signature(input: &str) -> (&str, Vec<&str>) {
    let (input, _) = identifier(input).unwrap();
    let (input, _) = multispace1::<&str, Error<_>>(input).unwrap();
    let (input, _) = identifier(input).unwrap();
    let (input, _) = multispace1::<&str, Error<_>>(input).unwrap();
    let (input, name) = identifier(input).unwrap();
    let (input, _) = multispace0::<&str, Error<_>>(input).unwrap();
    let (input, _) = tag::<&str, &str, Error<_>>("(")(input).unwrap();
    let (_, params) = separated_list0(
        char::<&str, Error<_>>(','),
        alt((take_until(","), take_until(")"))),
    )(input)
    .unwrap();

    let result_strings: Vec<&str> = if params.len() == 1 && params[0].is_empty() {
        vec![]
    } else {
        params.iter().map(|s| s.trim()).collect()
    };

    (name, result_strings)
}

fn parse_doc_comment(input: &str) -> IResult<&str, DocuComment> {
    let (input, _) = opt(multispace0)(input)?;
    let (input, description) = parse_description(input)?;
    let (input, _) = many0(parse_empty_line)(input)?;
    let (input, infos) = parse_infos(input)?;
    let (input, func) = parse_func(input)?;
    let func_str = func;
    let (name, _parameters) = parse_function_signature(&func_str);

    Ok((
        input,
        DocuComment {
            description,
            infos,
            func_string: func,
            func_name: name,
        },
    ))
}

fn parse_doc_comments(input: &str) -> IResult<&str, Vec<DocuComment>> {
    many0(delimited(multispace0, parse_doc_comment, multispace0))(input)
}

pub fn parse(input: &str) -> Option<String> {
    match parse_doc_comments(input).finish() {
        Ok(dcs) => Some(
            dcs.1
                .into_iter()
                .map(|s| s.generate_md())
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        Err(e) => {
            eprintln!("Someting went wrong {:#?}", e);
            None
        }
    }
}
