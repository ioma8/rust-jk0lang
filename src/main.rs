use core::fmt;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "jk0lang.pest"]
pub struct MyParser;

#[derive(Clone)]
enum ValueType {
    String,
    Float,
    Bool,
}

#[derive(Clone)]
struct Value {
    value_type: ValueType,
    value_string: String,
    value_float: f64,
    value_bool: bool,
    float_unit: String,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self.value_type {
            ValueType::String => self.value_string.clone(),
            ValueType::Float => self.value_float.to_string() + &self.float_unit,
            ValueType::Bool => self.value_bool.to_string(),
        };
        write!(f, "{}", string)
    }
}

struct MachineState {
    variables: Vec<(String, Value)>,
}

fn main() {
    println!("Parser of simple language called jk0lang");

    // read file into string
    let sample_code = std::fs::read_to_string("sample.jk0").expect("Unable to read file");

    // TODO: float value s jednotkou - kontrola na scitacich atd operacich, aby to umoznovalo jen soucet se stejjnou jednotkou a odecet
    // u nasobeni a deleni to jednotku upravi (lomitkem kk - cas pod a cas nad)

    // TODO: pridat do parseru podporu zakladnich matematichých operaci + - * / a závorek

    let file = MyParser::parse(Rule::main, &sample_code)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap();

    let mut machine_state = MachineState { variables: vec![] };

    for pair in file.into_inner() {
        match pair.as_rule() {
            Rule::function_call => {
                call_function(pair, &mut machine_state);
            }
            Rule::variable_declaration => {
                declare_variable(pair, &mut machine_state);
            }
            _ => {
                // println!("Other: {}", pair.as_str());
            }
        }
    }
}

fn declare_variable(pair: Pair<Rule>, machine_state: &mut MachineState) {
    let mut variable_name = "";
    let mut variable_value: Value = Value {
        value_type: ValueType::String,
        value_string: "".to_string(),
        value_float: 0.0,
        value_bool: false,
        float_unit: "".to_string(),
    };
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::symbol => {
                variable_name = inner_pair.as_str();
            }
            Rule::expression => {
                let variable_value_wrapped =
                    get_expression_value(inner_pair.into_inner().next().unwrap(), machine_state);
                match variable_value_wrapped {
                    Some(value) => {
                        variable_value = value;
                    }
                    None => {
                        return;
                    }
                }
            }
            _ => {
                //println!("Other: {}", inner_pair.as_str());
            }
        }
    }
    machine_state
        .variables
        .push((variable_name.to_string(), variable_value));
}

fn call_function(pair: Pair<Rule>, machine_state: &mut MachineState) {
    let mut function_name = "";
    let mut arguments: Vec<Value> = vec![];
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::symbol => {
                function_name = inner_pair.as_str();
            }
            Rule::expression => {
                let inner_pair = inner_pair.into_inner().next().unwrap();
                let value = get_expression_value(inner_pair, machine_state);
                if let Some(inner) = value {
                    arguments.push(inner);
                }
            }
            _ => {
                //println!("Other: {}", inner_pair.as_str());
            }
        }
    }
    match function_name {
        "println" => {
            println!(
                "{}",
                arguments
                    .into_iter()
                    .map(|it| { it.to_string() })
                    .collect::<Vec<_>>()
                    .join("")
            );
        }
        _ => {
            println!("Function not found: {}", function_name);
        }
    }
}

fn get_expression_value(pair: Pair<Rule>, machine_state: &mut MachineState) -> Option<Value> {
    match pair.as_rule() {
        Rule::symbol => {
            let variable_name = pair.as_str();
            let variable_value = get_variable_value(variable_name, machine_state);
            match variable_value {
                Some(value) => Some(value),
                None => {
                    println!("Variable not found: {}", variable_name);
                    None
                }
            }
        }
        Rule::value => {
            let value_parsed = parse_value(pair);
            Some(value_parsed)
        }
        _ => None,
    }
}

fn parse_value(pair: Pair<Rule>) -> Value {
    let mut value_type: ValueType = ValueType::String;
    let mut value_string = "";
    let mut value_float = 0.0;
    let mut value_bool = false;
    let mut float_unit = "";
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::string => {
                value_type = ValueType::String;
                value_string = inner_pair.as_str().trim_matches('"');
            }
            Rule::float => {
                value_type = ValueType::Float;
                for inner_inner_pair in inner_pair.into_inner() {
                    match inner_inner_pair.as_rule() {
                        Rule::float_value => {
                            value_float = inner_inner_pair
                                .as_str()
                                .replace(' ', "")
                                .parse::<f64>()
                                .unwrap();
                        }
                        Rule::float_unit => {
                            float_unit = inner_inner_pair.as_str();
                        }
                        _ => {}
                    }
                }
            }
            Rule::boolean => {
                value_type = ValueType::Bool;
                value_bool = inner_pair.as_str() == "true";
            }
            _ => {
                //println!("Other: {}", inner_pair.as_str());
            }
        }
    }
    Value {
        value_type,
        value_string: value_string.to_string(),
        value_float,
        value_bool,
        float_unit: float_unit.to_string(),
    }
}

fn get_variable_value(variable_name: &str, machine_state: &MachineState) -> Option<Value> {
    let variable = machine_state
        .variables
        .iter()
        .find(|(name, _)| name == variable_name);
    variable.map(|(_, value)| value.clone())
}
