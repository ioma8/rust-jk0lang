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
}

impl Value {
    fn to_string(&self) -> String {
        match self.value_type {
            ValueType::String => self.value_string.clone(),
            ValueType::Float => self.value_float.to_string(),
            ValueType::Bool => self.value_bool.to_string(),
        }
    }
}

struct MachineState {
    variables: Vec<(String, Value)>,
}

fn main() {
    println!("Parser of simple language called jk0lang");

    let sample_code = "
    val moja = \"moja hodnota\"
    println(\"tisknu text\")
    println(moja)
    println(123)
    println(true)
    val cislo = 123
    println(cislo)
    println(nedefinovano)
    neznama_fn()
    val tvoja = moja
    println(tvoja)
    val chybejici = xxyyzz
    val floating = 1.23
    println(\"floating value: \", floating)
    println(\"negative float value: \", -1.23)
    ";

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
                match value {
                    Some(value) => {
                        arguments.push(value);
                    }
                    None => {}
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
            return Some(value_parsed);
        }
        _ => {
            return None;
        }
    }
}

fn parse_value(pair: Pair<Rule>) -> Value {
    let mut value_type: ValueType = ValueType::String;
    let mut value_string = "";
    let mut value_float = 0.0;
    let mut value_bool = false;
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::string => {
                value_type = ValueType::String;
                value_string = inner_pair.as_str().trim_matches('"');
            }
            Rule::float => {
                value_type = ValueType::Float;
                value_float = inner_pair.as_str().parse::<f64>().unwrap();
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
    }
}

fn get_variable_value(variable_name: &str, machine_state: &MachineState) -> Option<Value> {
    let variable = machine_state
        .variables
        .iter()
        .find(|(name, _)| name == variable_name);
    match variable {
        Some((_, value)) => Some(value.clone()),
        None => None,
    }
}
