const INPUT: &str = r#"
/// Sets up the visual of an NPC
///
/// @param npc NPC to be affected
/// @param body_mesh mesh to be used as the body e.g. `HUN_BODY_NAKED0`
/// @param body_tex body texture assigned to this body mesh
/// @param skin body texture variant
/// @param head_mesh head mesh
/// @param head_tex head texture
/// @param teeth_tex teeth texture
/// @param armor_inst armor (C_ITEM instance) to be equipped or `-1` for no armor
func void Mdl_SetVisualBody( var instance npc,
                            var string body_mesh,
                            var int body_tex,
                            var int skin,
                            var string head_mesh,
                            var int head_tex,
                            var int teeth_tex,
                            var int armor_inst ) {};


/// Display the document using the document manager ID
///
/// @param docID document manager ID
func void Doc_Show(var int docID) {};


/// Create a new instance of the document manager with the arrow showing players position on the map and returns its ID.
///
/// @return Returns the ID of the document manager instance.
func int Doc_CreateMap() {};
"#;

use std::process::exit;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{
        alpha1, alphanumeric1, char, line_ending, multispace0, multispace1, not_line_ending,
    },
    combinator::{opt, recognize},
    error::Error,
    multi::{many0, many0_count, separated_list0},
    sequence::{delimited, pair},
    Finish, IResult,
};

#[derive(Debug)]
struct DocuComment {
    description: Option<String>,
    param_desc: Option<Vec<(String, String)>>,
    parameters: Vec<String>,
    ret_stmt: Option<String>,
    func_string: String,
    func_name: String,
}

impl DocuComment {
    pub fn generate_md(&self) -> String {
        let mut md = String::with_capacity(50);
        md.push_str(&format!("### `{}`\n", self.func_name));
        md.push_str(&format!("!!! function \"`{}`\"\n", self.func_name));
        if let Some(desc) = &self.description {
            md.push_str(&format!("\t{}\n", desc));
        }
        md.push_str(&format!("\t```dae\n\t{}\n\t```\n", self.func_string));
        if let Some(params) = &self.param_desc {
            if !params.is_empty() {
                md.push_str("\n\t**Parameters**  \n");
            }
            for (i, p) in params.iter().enumerate() {
                md.push_str(&format!("\t- `#!dae {}` - {}\n", self.parameters[i], p.1))
            }
        }
        if let Some(ret) = &self.ret_stmt {
            md.push_str("\n\t**Return value**  \n");
            md.push_str(&format!("\t{}\n", ret));
        }
        // md.push_str(&format!("\n"));
        // "".to_string()
        md
    }
}

fn parse_description(input: &str) -> IResult<&str, Option<String>> {
    let (input, desc) = alt((take_until("///\n"), take_until("func")))(input)?;
    let desc = desc
        .strip_prefix("///")
        .expect("this string to start with ///")
        .trim();
    if desc.is_empty() {
        Ok((input, None))
    } else {
        Ok((input, Some(desc.to_string())))
    }
}

fn parse_empty_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("///")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, ()))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_func(input: &str) -> IResult<&str, String> {
    let (input, func) = take_until("{};")(input)?;
    let (input, _) = tag("{};")(input)?;
    Ok((input, format!("{} {{}};", func)))
}

// fn parse_func_name(input: &str) -> String {
//     let (input, _) = identifier(input).unwrap();
//     let (input, _) = multispace1::<&str, Error<_>>(input).unwrap();
//     let (input, _) = identifier(input).unwrap();
//     let (input, _) = multispace1::<&str, Error<_>>(input).unwrap();
//     let (_, name) = identifier(input).unwrap();
//     name.to_string()
// }

fn parse_function_signature(input: &str) -> (String, Vec<String>) {
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

    // println!("parms.len(): {}", params.len());
    // println!("parms: {:#?}", params);
    let result_strings: Vec<String> = if params.len() == 1 && params[0].is_empty() {
        vec![]
    } else {
        params.iter().map(|s| s.trim().to_string()).collect()
    };

    (name.to_string(), result_strings)
}

fn parse_param(input: &str) -> IResult<&str, (String, String)> {
    let (input, _) = tag("/// @param ")(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = multispace1(input)?;
    let (input, value) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, (name.to_string(), value.trim().to_string())))
}

fn parse_return(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("/// @return ")(input)?;
    let (input, ret) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ret.trim().to_string()))
}

fn parse_doc_comment(input: &str) -> IResult<&str, DocuComment> {
    let (input, _) = opt(multispace0)(input)?;
    let (input, description) = parse_description(input)?;
    let (input, _) = parse_empty_line(input)?;
    let (input, params) = many0(parse_param)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, ret_stmt) = opt(parse_return)(input)?;
    let (input, func) = parse_func(input)?;
    // let name = parse_func_name(&func.clone());
    let func_str = func.clone();
    let (name, parameters) = parse_function_signature(&func_str);
    // println!("parameters: {:#?}", parameters);
    // println!(" - params: {}\n - parameters: {}", params.len(), parameters.len());

    Ok((
        input,
        DocuComment {
            description,
            param_desc: Some(params),
            parameters,
            ret_stmt,
            func_string: func,
            func_name: name.to_string(),
        },
    ))
}

fn parse_doc_comments(input: &str) -> IResult<&str, Vec<DocuComment>> {
    many0(delimited(multispace0, parse_doc_comment, multispace0))(input)
}

fn parse(input: &str) -> Option<String> {
    match parse_doc_comments(input).finish() {
        Ok(dcs) => {
            return Some(
                dcs.1
                    .into_iter()
                    .map(|s| s.generate_md())
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        }
        Err(e) => {
            eprintln!("Someting went wrong {:#?}", e);
            exit(1);
        }
    }
}

fn main() {
    // let result = parse_doc_comment(INPUT);
    let result = parse(INPUT);

    // println!("{:#?}", result.as_ref().unwrap().1);
    if let Some(res) = result {
        println!("{}", res);
    }
}
